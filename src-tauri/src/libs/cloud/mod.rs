use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use std::path::PathBuf;

pub mod dropbox;
pub mod models;

// Re-export commonly used types
pub use models::{CloudProvider as CloudProviderModel, CloudFolder as CloudFolderModel, CloudSync};
pub use models::{
    SYNC_STATUS_SYNCED,
    SYNC_STATUS_PENDING_UPLOAD,
    SYNC_STATUS_PENDING_DOWNLOAD,
    SYNC_STATUS_CONFLICT,
    ITEM_TYPE_TRACK,
    ITEM_TYPE_PLAYLIST,
};

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudFile {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub size: u64,
    pub is_folder: bool,
    pub modified_at: i64,
    pub created_at: i64,
    pub mime_type: Option<String>,
    pub hash: Option<FileHash>,        // Content hash for change detection
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub enum FileHash {
    Sha1(String),
    Sha256(String),
    ContentHash(String), // For Dropbox
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudAuth {
    pub provider_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub auth_data: Option<String>,
}

#[async_trait]
pub trait CloudProvider: Send + Sync {
    // Basic provider info
    fn provider_type(&self) -> &'static str;
    
    // Authorization
    async fn is_authorized(&self) -> bool;
    async fn unauthorize(&self);
    
    // File operations
    async fn list_files(&self, folder_id: &str) -> Result<Vec<CloudFile>, String>;
    async fn list_files_recursive(&self, folder_id: &str) -> Result<Vec<CloudFile>, String>;
    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<CloudFile, String>;
    async fn upload_file(&self, local_path: &PathBuf, name: &str, parent_id: Option<&str>) -> Result<CloudFile, String>;
    async fn download_file(&self, file_id: &str, local_path: &PathBuf) -> Result<(), String>;
    async fn delete_file(&self, file_id: &str) -> Result<(), String>;
}

