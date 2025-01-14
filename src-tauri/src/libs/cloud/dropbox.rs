use std::path::PathBuf;
use async_trait::async_trait;
use tokio::sync::Mutex;
use dropbox_sdk::{
    default_client::{NoauthDefaultClient, UserAuthDefaultClient},
    oauth2::{Authorization, AuthorizeUrlBuilder, Oauth2Type, PkceCode},
    files::{self, ListFolderArg, ListFolderResult, FileMetadata, FolderMetadata, CreateFolderArg},
};
use log::{error, info, warn};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::io::Read;
use mime_guess::from_path;

use super::{CloudProvider, CloudFile, CloudAuth, FileHash};

const DROPBOX_CLIENT_ID: &str = "jgibk23zkucv2ec";
const PROVIDER_TYPE: &str = "dropbox";

pub struct Dropbox {
    pkce_code: Mutex<Option<PkceCode>>,
    authorization: Mutex<Option<Authorization>>,
    client: Mutex<Option<UserAuthDefaultClient>>,
}

impl Dropbox {
    pub fn new() -> Self {
        Self {
            pkce_code: Mutex::new(None),
            authorization: Mutex::new(None),
            client: Mutex::new(None),
        }
    }

    fn folder_id_to_path(&self, folder_id: &str) -> String {
        if folder_id.is_empty() || folder_id == "/" {
            String::new()
        } else {
            folder_id.to_string()
        }
    }

    fn get_parent_id(path: &str) -> Option<String> {
        if path.is_empty() {
            None
        } else {
            let parent = std::path::Path::new(path)
                .parent()
                .and_then(|p| p.to_str())
                .map(|s| if s == "/" { String::new() } else { s.to_string() });
            parent
        }
    }

    async fn list_files(&self, folder_id: &str, recursive: bool) -> Result<Vec<CloudFile>, String> {
        let path = self.folder_id_to_path(folder_id);
        let list_folder_arg = files::ListFolderArg::new(path)
            .with_recursive(recursive)
            .with_include_media_info(true)
            .with_include_deleted(false);

        let client = self.client.lock().await;
        let client_ref = client.as_ref().ok_or("Not authorized")?;

        let result = files::list_folder(client_ref, &list_folder_arg)
            .map_err(|e| {
                error!("Failed to list Dropbox files: {}", e);
                format!("Failed to list Dropbox files: {}", e)
            })?;

        let cloud_files = result
            .entries
            .par_iter()
            .filter_map(|entry| {
                match entry {
                    files::Metadata::File(f) => {
                        let modified_at = DateTime::parse_from_rfc3339(&f.server_modified)
                            .map(|dt| dt.timestamp())
                            .unwrap_or(0);
                        Some(CloudFile {
                            id: f.id.clone(),
                            name: f.name.clone(),
                            parent_id: Self::get_parent_id(&f.path_display.clone().unwrap_or_default()),
                            size: f.size as u64,
                            is_folder: false,
                            modified_at,
                            created_at: modified_at,
                            mime_type: Some(from_path(&f.name).first_or_octet_stream().to_string()),
                            hash: f.content_hash.as_ref().map(|h| FileHash::ContentHash(h.clone())),
                        })
                    }
                    files::Metadata::Folder(f) => Some(CloudFile {
                        id: f.id.clone(),
                        name: f.name.clone(),
                        parent_id: Self::get_parent_id(&f.path_display.clone().unwrap_or_default()),
                        size: 0,
                        is_folder: true,
                        modified_at: 0,
                        created_at: 0,
                        mime_type: None,
                        hash: None,
                    }),
                    _ => None,
                }
            })
            .collect::<Vec<CloudFile>>();
        Ok(cloud_files)
    }

    pub async fn start_authorization(&self) -> Result<String, String> {
        info!("Generating Dropbox authorization URL");
        let pkce_code = PkceCode::new();
        let mut pkce_code_guard = self.pkce_code.lock().await;
        *pkce_code_guard = Some(pkce_code.clone());
        let flow_type = Oauth2Type::PKCE(pkce_code.clone());
        let auth_url = AuthorizeUrlBuilder::new(DROPBOX_CLIENT_ID, &flow_type).build();

        info!("Generated authorization URL successfully");
        Ok(auth_url.to_string())
    }

    pub async fn complete_authorization(&self, auth_code: &str) -> Result<CloudAuth, String> {
        info!("Completing Dropbox authorization");

        let pkce_code = self.pkce_code.lock().await.take().ok_or_else(|| {
            error!("No PKCE code found in state");
            "No PKCE code found. Please start the authorization process again.".to_string()
        })?;

        let flow_type: Oauth2Type = Oauth2Type::PKCE(pkce_code);

        let mut auth = Authorization::from_auth_code(
            DROPBOX_CLIENT_ID.to_string(),
            flow_type,
            auth_code.to_string(),
            None,
        );

        info!("Obtaining access token...");
        let client = NoauthDefaultClient::default();
        let access_token = auth.obtain_access_token(client).map_err(|e| {
            error!("Failed to obtain access token: {}", e);
            format!("Failed to obtain access token: {}", e)
        })?;

        let mut auth_guard = self.authorization.lock().await;
        *auth_guard = Some(auth.clone());

        let client = UserAuthDefaultClient::new(auth.clone());
        self.client.lock().await.replace(client);

        let auth_data = auth.save();
        info!("Authorization completed successfully");

        Ok(CloudAuth {
            provider_id: Uuid::new_v4().to_string(),
            access_token,
            refresh_token: None,
            auth_data: auth_data,
        })
    }
}

#[async_trait]
impl CloudProvider for Dropbox {
    fn provider_type(&self) -> &'static str {
        PROVIDER_TYPE
    }

    async fn is_authorized(&self) -> bool {
        self.authorization.lock().await.is_some() && self.client.lock().await.is_some()
    }

    async fn unauthorize(&self) {
        let mut auth_guard = self.authorization.lock().await;
        *auth_guard = None;
        let mut client_guard = self.client.lock().await;
        *client_guard = None;
    }

    async fn list_files(&self, folder_id: &str) -> Result<Vec<CloudFile>, String> {
        self.list_files(folder_id, false).await
    }

    async fn list_files_recursive(&self, folder_id: &str) -> Result<Vec<CloudFile>, String> {
        self.list_files(folder_id, true).await
    }

    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<CloudFile, String> {
        let client = self.client.lock().await;
        let client_ref = client.as_ref().ok_or("Not authorized")?;

        let parent_path = parent_id.map(|id| self.folder_id_to_path(id)).unwrap_or_default();
        let folder_path = if parent_path.is_empty() {
            format!("/{}", name)
        } else {
            format!("{}/{}", parent_path, name)
        };

        let create_folder_arg = files::CreateFolderArg::new(folder_path);
        let result = files::create_folder_v2(client_ref, &create_folder_arg)
            .map_err(|e| {
                error!("Failed to create Dropbox folder: {}", e);
                format!("Failed to create Dropbox folder: {}", e)
            })?;

        Ok(CloudFile {
            id: result.metadata.id,
            name: result.metadata.name,
            parent_id: parent_id.map(String::from),
            size: 0,
            is_folder: true,
            modified_at: 0,
            created_at: 0,
            mime_type: None,
            hash: None,
        })
    }

    async fn upload_file(&self, local_path: &PathBuf, name: &str, parent_id: Option<&str>) -> Result<CloudFile, String> {
        let client = self.client.lock().await;
        let client_ref = client.as_ref().ok_or("Not authorized")?;

        let parent_path = parent_id.map(|id| self.folder_id_to_path(id)).unwrap_or_default();
        let file_path = if parent_path.is_empty() {
            format!("/{}", name)
        } else {
            format!("{}/{}", parent_path, name)
        };

        let file_content = std::fs::read(local_path).map_err(|e| {
            error!("Failed to read local file: {}", e);
            format!("Failed to read local file: {}", e)
        })?;

        let upload_arg = files::UploadArg::new(file_path)
            .with_mode(files::WriteMode::Overwrite);

        let result = files::upload(client_ref, &upload_arg, file_content.as_ref())
            .map_err(|e| {
                error!("Failed to upload file: {}", e);
                format!("Failed to upload file: {}", e)
        })?;

        let modified_at = DateTime::parse_from_rfc3339(&result.server_modified)
            .map(|dt| dt.timestamp())
            .unwrap_or(0);

        Ok(CloudFile {
            id: result.id,
            name: result.name.clone(),
            parent_id: parent_id.map(String::from),
            size: result.size as u64,
            is_folder: false,
            modified_at,
            created_at: modified_at,
            mime_type: Some(from_path(&result.name).first_or_octet_stream().to_string()),
            hash: result.content_hash.as_ref().map(|h| FileHash::ContentHash(h.clone())),
        })
    }

    async fn download_file(&self, file_id: &str, local_path: &PathBuf) -> Result<(), String> {
        let client = self.client.lock().await;
        let client_ref = client.as_ref().ok_or("Not authorized")?;

        let download_arg = files::DownloadArg::new(file_id.to_string());
        let result = files::download(client_ref, &download_arg, None, None)
            .map_err(|e| {
                error!("Failed to download file: {}", e);
                format!("Failed to download file: {}", e)
            })?;

        let mut buffer = Vec::new();
        result.body.unwrap().read_to_end(&mut buffer).map_err(|e| {
            error!("Failed to read file content: {}", e);
            format!("Failed to read file content: {}", e)
        })?;

        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                error!("Failed to create parent directory: {}", e);
                format!("Failed to create parent directory: {}", e)
            })?;
        }

        std::fs::write(local_path, buffer).map_err(|e| {
            error!("Failed to write local file: {}", e);
            format!("Failed to write local file: {}", e)
        })?;

        Ok(())
    }

    async fn delete_file(&self, file_id: &str) -> Result<(), String> {
        let client = self.client.lock().await;
        let client_ref = client.as_ref().ok_or("Not authorized")?;

        let delete_arg = files::DeleteArg::new(file_id.to_string());
        files::delete_v2(client_ref, &delete_arg)
            .map_err(|e| {
                error!("Failed to delete file: {}", e);
                format!("Failed to delete file: {}", e)
            })?;

        Ok(())
    }
}
