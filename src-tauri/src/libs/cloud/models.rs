use ormlite::model::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

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