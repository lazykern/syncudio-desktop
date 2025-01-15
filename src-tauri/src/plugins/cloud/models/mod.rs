use ormlite::model::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use async_trait::async_trait;
use std::path::PathBuf;

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
    pub hash: Option<FileHash>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub enum FileHash {
    Sha1(String),
    Sha256(String),
    ContentHash(String), // For Dropbox
}

#[async_trait]
pub trait CloudProvider {
    async fn is_authorized(&self) -> bool;
    async fn unauthorize(&self);
    async fn list_files(&self, folder_id: &str, recursive: bool) -> Result<Vec<CloudFile>, String>;
    async fn list_root_files(&self, recursive: bool) -> Result<Vec<CloudFile>, String>;
    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<CloudFile, String>;
    async fn upload_file(&self, local_path: &PathBuf, name: &str, parent_id: Option<&str>) -> Result<CloudFile, String>;
    async fn download_file(&self, file_id: &str, local_path: &PathBuf) -> Result<(), String>;
    async fn delete_file(&self, file_id: &str) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "cloud_folders")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudFolder {
    #[ormlite(primary_key)]
    pub id: String,
    pub provider_type: String,
    pub cloud_folder_id: String,
    pub cloud_folder_name: String,
    pub local_folder_path: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "cloud_syncs")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudSync {
    #[ormlite(primary_key)]
    pub id: String,
    pub provider_type: String,
    pub folder_id: String,
    pub item_id: String,
    pub item_type: String, // "track" or "playlist"
    pub cloud_file_id: String,
    pub cloud_file_name: String,
    pub local_path: String,
    pub last_synced: Option<i64>,
    pub sync_status: String, // "synced", "pending_upload", "pending_download", "conflict"
    pub created_at: i64,
    pub updated_at: i64,
}

impl CloudFolder {
    pub fn new(
        id: String,
        provider_type: String,
        cloud_folder_id: String,
        cloud_folder_name: String,
        local_folder_path: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            provider_type,
            cloud_folder_id,
            cloud_folder_name,
            local_folder_path,
            created_at: now,
            updated_at: now,
        }
    }
}

impl CloudSync {
    pub fn new(
        id: String,
        provider_type: String,
        folder_id: String,
        item_id: String,
        item_type: String,
        cloud_file_id: String,
        cloud_file_name: String,
        local_path: String,
        sync_status: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            provider_type,
            folder_id,
            item_id,
            item_type,
            cloud_file_id,
            cloud_file_name,
            local_path,
            last_synced: None,
            sync_status,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn mark_synced(&mut self) {
        self.last_synced = Some(chrono::Utc::now().timestamp());
        self.sync_status = "synced".to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn mark_pending_upload(&mut self) {
        self.sync_status = "pending_upload".to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn mark_pending_download(&mut self) {
        self.sync_status = "pending_download".to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn mark_conflict(&mut self) {
        self.sync_status = "conflict".to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

// Constants for sync status
pub const SYNC_STATUS_SYNCED: &str = "synced";
pub const SYNC_STATUS_PENDING_UPLOAD: &str = "pending_upload";
pub const SYNC_STATUS_PENDING_DOWNLOAD: &str = "pending_download";
pub const SYNC_STATUS_CONFLICT: &str = "conflict";

// Constants for item types
pub const ITEM_TYPE_TRACK: &str = "track";
pub const ITEM_TYPE_PLAYLIST: &str = "playlist"; 