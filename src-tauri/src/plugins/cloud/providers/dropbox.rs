use async_trait::async_trait;
use chrono::DateTime;
use dropbox_sdk::{
    default_client::{NoauthDefaultClient, UserAuthDefaultClient},
    files::{self},
    oauth2::{Authorization, AuthorizeUrlBuilder, Oauth2Type, PkceCode},
};
use log::info;
use mime_guess::from_path;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use tokio::sync::Mutex;

use crate::plugins::cloud::providers::CloudProviderType;
use crate::plugins::cloud::CloudProvider;
use crate::plugins::cloud::FileHash;
use crate::plugins::config::get_storage_dir;
use crate::{
    libs::error::{AnyResult, SyncudioError},
    plugins::cloud::CloudFile,
};

const DROPBOX_CLIENT_ID: &str = "jgibk23zkucv2ec";

type DropboxAuthData = Option<String>;

pub struct Dropbox {
    pkce_code: Mutex<Option<PkceCode>>,
    authorization: Mutex<Option<Authorization>>,
    client: Mutex<Option<UserAuthDefaultClient>>,
}

impl Dropbox {
    pub fn new() -> Self {
        let auth_data = Self::load_auth_data_from_file();
        if let Some(auth_data) = auth_data {
            if let Some(dropbox) = Self::new_with_auth_data(auth_data) {
                return dropbox;
            }
        }

        Self {
            pkce_code: Mutex::new(None),
            authorization: Mutex::new(None),
            client: Mutex::new(None),
        }
    }

    pub fn provider_type() -> CloudProviderType {
        CloudProviderType::Dropbox
    }

    fn get_auth_file_path() -> PathBuf {
        get_storage_dir().join("dropbox_auth.dat")
    }

    fn load_auth_data_from_file() -> Option<String> {
        let path = Self::get_auth_file_path();
        fs::read_to_string(path).ok()
    }

    fn save_auth_data_to_file(auth_data: &str) -> AnyResult<()> {
        let path = Self::get_auth_file_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, auth_data)?;
        Ok(())
    }

    pub fn new_with_auth_data(auth_data: String) -> Option<Self> {
        let auth = Authorization::load(DROPBOX_CLIENT_ID.to_string(), &auth_data);

        match auth {
            Some(auth) => {
                let client = UserAuthDefaultClient::new(auth.clone());
                Some(Self {
                    pkce_code: Mutex::new(None),
                    authorization: Mutex::new(Some(auth)),
                    client: Mutex::new(Some(client)),
                })
            }
            None => None,
        }
    }

    pub async fn start_authorization(&self) -> AnyResult<String> {
        info!("Generating Dropbox authorization URL");
        let pkce_code = PkceCode::new();
        let mut pkce_code_guard = self.pkce_code.lock().await;
        *pkce_code_guard = Some(pkce_code.clone());
        let flow_type = Oauth2Type::PKCE(pkce_code.clone());
        let auth_url = AuthorizeUrlBuilder::new(DROPBOX_CLIENT_ID, &flow_type).build();

        info!("Generated authorization URL successfully");
        Ok(auth_url.to_string())
    }

    pub async fn complete_authorization(&self, auth_code: &str) -> AnyResult<DropboxAuthData> {
        info!("Completing Dropbox authorization");

        let pkce_code = self.pkce_code.lock().await.take().ok_or_else(|| {
            SyncudioError::Dropbox(
                "No PKCE code found. Please start the authorization process again.".to_string(),
            )
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
        let access_token = auth.obtain_access_token(client)?;

        let mut auth_guard = self.authorization.lock().await;
        *auth_guard = Some(auth.clone());

        let client = UserAuthDefaultClient::new(auth.clone());
        self.client.lock().await.replace(client);

        let auth_data = auth.save();
        info!("Authorization data saved successfully: {:?}", auth_data);

        // Save auth data to file
        if let Some(auth_data_str) = &auth_data {
            Self::save_auth_data_to_file(auth_data_str)?;
        }

        Ok(auth_data)
    }

    fn amend_path_or_id(&self, folder_id: &str) -> String {
        if folder_id.is_empty() || folder_id == "/" {
            String::new()
        } else {
            folder_id.to_string()
        }
    }
}

#[async_trait]
impl CloudProvider for Dropbox {
    fn provider_type(&self) -> CloudProviderType {
        CloudProviderType::Dropbox
    }

    async fn is_authorized(&self) -> bool {
        self.authorization.lock().await.is_some() && self.client.lock().await.is_some()
    }

    async fn unauthorize(&self) {
        let mut auth_guard = self.authorization.lock().await;
        *auth_guard = None;
        let mut client_guard = self.client.lock().await;
        *client_guard = None;

        // Remove auth file
        let _ = fs::remove_file(Self::get_auth_file_path());
    }

    async fn list_files(&self, folder_id: &str, folder_path: &str, recursive: bool) -> AnyResult<Vec<CloudFile>> {
        let path_or_id = self.amend_path_or_id(folder_id);
        let list_folder_arg = files::ListFolderArg::new(path_or_id)
            .with_recursive(recursive)
            .with_include_media_info(true)
            .with_include_deleted(false);

        let client = self.client.lock().await;
        let client_ref = client.as_ref().ok_or(SyncudioError::Dropbox("Not authorized".to_string()))?;
        let result = files::list_folder(client_ref, &list_folder_arg)?;

        info!("Found {} files in Dropbox", result.entries.len());

        let cloud_files = result.entries.par_iter().filter_map(|entry| match entry {
            files::Metadata::File(f) => Some(CloudFile {
                id: f.id.clone(),
                name: f.name.clone(),
                size: f.size as u64,
                is_folder: false,
                modified_at: DateTime::parse_from_rfc3339(&f.server_modified).unwrap_or_default().into(),
                mime_type: Some(from_path(&f.name).first_or_octet_stream().to_string()),
                hash: f.content_hash.as_ref().map(|h| FileHash::ContentHash(h.clone())),
                display_path: f.path_display.clone(),
                relative_path: f.path_display.clone().unwrap_or_default().strip_prefix(folder_path).unwrap_or_default().to_string(),
            }),
            files::Metadata::Folder(f) => Some(CloudFile {
                id: f.id.clone(),
                name: f.name.clone(),
                size: 0,
                is_folder: true,
                modified_at: DateTime::from_timestamp(0, 0).unwrap_or_default(),
                mime_type: None,
                hash: None,
                display_path: f.path_display.clone(),
                relative_path: f.path_display.clone().unwrap_or_default().strip_prefix(folder_path).unwrap_or_default().to_string(),
            }),
            _ => None,
        }).collect();

        Ok(cloud_files)
    }

    async fn list_root_files(&self, recursive: bool) -> AnyResult<Vec<CloudFile>> {
        self.list_files("", "/", recursive).await
    }

    async fn create_folder(&self, name: &str, parent_ref: Option<&str>) -> AnyResult<CloudFile> {
        let client = self.client.lock().await;
        let client_ref = client
            .as_ref()
            .ok_or(SyncudioError::Dropbox("Not authorized".to_string()))?;

        // For Dropbox, parent_ref is already a path
        let folder_path = match parent_ref {
            Some(path) if !path.is_empty() => format!("{}/{}", path, name),
            _ => format!("/{}", name),
        };

        let create_folder_arg = files::CreateFolderArg::new(folder_path.clone());
        let result = files::create_folder_v2(client_ref, &create_folder_arg)?;

        Ok(CloudFile {
            id: result.metadata.id,
            name: result.metadata.name,
            size: 0,
            is_folder: true,
            modified_at: DateTime::from_timestamp(0, 0).unwrap_or_default(),
            mime_type: None,
            hash: None,
            display_path: result.metadata.path_display,
            relative_path: name.to_string(),
        })
    }

    async fn upload_file(
        &self,
        local_path: &PathBuf,
        name: &str,
        parent_ref: Option<&str>,
    ) -> AnyResult<CloudFile> {
        let client = self.client.lock().await;
        let client_ref = client
            .as_ref()
            .ok_or(SyncudioError::Dropbox("Not authorized".to_string()))?;

        // For Dropbox, parent_ref is already a path
        let file_path = match parent_ref {
            Some(path) if !path.is_empty() => format!("{}/{}", path, name),
            _ => format!("/{}", name),
        };

        let file_content = std::fs::read(local_path)?;
        let upload_arg = files::UploadArg::new(file_path.clone()).with_mode(files::WriteMode::Overwrite);
        let result = files::upload(client_ref, &upload_arg, file_content.as_ref())?;

        let modified_at = DateTime::parse_from_rfc3339(&result.server_modified)
            .unwrap_or_default();

        Ok(CloudFile {
            id: result.id,
            name: result.name.clone(),
            size: result.size as u64,
            is_folder: false,
            modified_at: modified_at.into(),
            mime_type: Some(from_path(&result.name).first_or_octet_stream().to_string()),
            hash: result
                .content_hash
                .as_ref()
                .map(|h| FileHash::ContentHash(h.clone())),
            display_path: result.path_display,
            relative_path: name.to_string(),
        })
    }

    async fn download_file(&self, file_id: &str, local_path: &PathBuf) -> AnyResult<()> {
        let client = self.client.lock().await;
        let client_ref = client
            .as_ref()
            .ok_or(SyncudioError::Dropbox("Not authorized".to_string()))?;

        let download_arg = files::DownloadArg::new(file_id.to_string());
        let result = files::download(client_ref, &download_arg, None, None)?;

        let mut buffer = Vec::new();
        result
            .body
            .ok_or(SyncudioError::Dropbox(
                "Failed to read file content".to_string(),
            ))?
            .read_to_end(&mut buffer)?;

        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(local_path, buffer)?;
        Ok(())
    }

    async fn delete_file(&self, file_id: &str) -> AnyResult<()> {
        let client = self.client.lock().await;
        let client_ref = client
            .as_ref()
            .ok_or(SyncudioError::Dropbox("Not authorized".to_string()))?;

        let delete_arg = files::DeleteArg::new(file_id.to_string());
        files::delete_v2(client_ref, &delete_arg)?;
        Ok(())
    }
}
