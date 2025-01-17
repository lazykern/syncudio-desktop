use ormlite::model::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::libs::error::{AnyResult, SyncudioError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub enum SyncQueueStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl SyncQueueStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncQueueStatus::Pending => "pending",
            SyncQueueStatus::InProgress => "in_progress",
            SyncQueueStatus::Completed => "completed",
            SyncQueueStatus::Failed => "failed",
            SyncQueueStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> AnyResult<Self> {
        match s {
            "pending" => Ok(SyncQueueStatus::Pending),
            "in_progress" => Ok(SyncQueueStatus::InProgress),
            "completed" => Ok(SyncQueueStatus::Completed),
            "failed" => Ok(SyncQueueStatus::Failed),
            "cancelled" => Ok(SyncQueueStatus::Cancelled),
            _ => Err(SyncudioError::InvalidQueueStatus),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "download_queue")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct DownloadQueueItem {
    #[ormlite(primary_key)]
    pub id: String,
    pub cloud_track_path_id: String,
    pub provider_type: String,
    pub size: u32,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub attempts: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "upload_queue")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct UploadQueueItem {
    #[ormlite(primary_key)]
    pub id: String,
    pub cloud_track_path_id: String,
    pub provider_type: String,
    pub size: u32,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub attempts: i32,
} 