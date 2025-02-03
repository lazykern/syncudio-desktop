mod dropbox;

use chrono::{DateTime, Utc};
pub use dropbox::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::libs::error::{AnyResult, SyncudioError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub enum CloudProviderType {
    #[serde(rename = "dropbox")]
    Dropbox,
    #[serde(rename = "gdrive")]
    GoogleDrive,
}

impl CloudProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CloudProviderType::Dropbox => "dropbox",
            CloudProviderType::GoogleDrive => "gdrive",
        }
    }

    pub fn from_str(s: &str) -> AnyResult<Self> {
        match s {
            "dropbox" => Ok(CloudProviderType::Dropbox),
            "gdrive" => Ok(CloudProviderType::GoogleDrive),
            _ => Err(SyncudioError::InvalidProviderType),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudFile {
    pub id: String,
    pub name: String,
    pub size: u32,
    pub is_folder: bool,
    pub modified_at: DateTime<Utc>,
    pub mime_type: Option<String>,
    pub hash: Option<FileHash>,
    pub display_path: Option<String>,
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub enum FileHash {
    Sha1(String),
    Sha256(String),
    ContentHash(String), // For Dropbox
}

use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait CloudProvider {
    async fn is_authorized(&self) -> bool;
    async fn unauthorize(&self);
    async fn list_files(&self, folder_id: &str, folder_path: &str, recursive: bool) -> AnyResult<Vec<CloudFile>>;
    async fn list_root_files(&self, recursive: bool) -> AnyResult<Vec<CloudFile>>;
    async fn create_folder(&self, name: &str, parent_ref: Option<&str>) -> AnyResult<CloudFile>;
    async fn upload_file(&self, local_path: &PathBuf, name: &str, parent_ref: Option<&str>) -> AnyResult<CloudFile>;
    async fn download_file(&self, file_id: &str, local_path: &PathBuf) -> AnyResult<()>;
    async fn delete_file(&self, file_id: &str) -> AnyResult<()>;

    // Get the full path or ID for a parent reference based on provider
    fn get_parent_ref(&self, parent_id: Option<&str>, parent_path: Option<&str>) -> Option<String> {
        match self.provider_type() {
            CloudProviderType::Dropbox => parent_path.map(|p| p.to_string()),
            CloudProviderType::GoogleDrive => parent_id.map(|id| id.to_string()),
        }
    }

    // Get provider type
    fn provider_type(&self) -> CloudProviderType;

    // Metadata sync methods
    async fn ensure_metadata_folder(&self) -> AnyResult<String> {
        // Create /Syncudio if it doesn't exist
        let syncudio = match self.list_root_files(false).await?.iter()
            .find(|f| f.is_folder && f.name == "Syncudio") {
                Some(f) => f.id.clone(),
                None => {
                    let parent_ref = self.get_parent_ref(None, Some("/"));
                    self.create_folder("Syncudio", parent_ref.as_deref()).await?.id
                }
            };

        // Create /Syncudio/metadata if it doesn't exist
        let metadata = match self.list_files(&syncudio, "/Syncudio", false).await?.iter()
            .find(|f| f.is_folder && f.name == "metadata") {
                Some(f) => Ok(f.id.clone()),
                None => {
                    let parent_ref = self.get_parent_ref(
                        Some(&syncudio),
                        Some("/Syncudio")
                    );
                    Ok(self.create_folder("metadata", parent_ref.as_deref()).await?.id)
                }
            };
        metadata
    }
} 
