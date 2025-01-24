use chrono::{DateTime, Utc};
use ormlite::model::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

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
    pub priority: i32,
    pub cloud_map_id: String,
    pub provider_type: String,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub attempts: i32,
}

impl DownloadQueueItem {
    pub fn new(cloud_map_id: String, provider_type: String, priority: i32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            priority,
            cloud_map_id,
            provider_type,
            status: SyncQueueStatus::Pending.as_str().to_string(),
            error_message: None,
            created_at: now,
            updated_at: now,
            attempts: 0,
        }
    }

    pub fn get_status(&self) -> AnyResult<SyncQueueStatus> {
        SyncQueueStatus::from_str(&self.status)
    }

    pub fn set_status(&mut self, status: SyncQueueStatus) {
        self.status = status.as_str().to_string();
        self.updated_at = Utc::now();
    }

    pub fn start_processing(&mut self) {
        self.set_status(SyncQueueStatus::InProgress);
    }

    pub fn complete(&mut self) {
        self.set_status(SyncQueueStatus::Completed);
    }

    pub fn fail(&mut self, error: String) {
        self.error_message = Some(error);
        self.attempts += 1;
        self.set_status(SyncQueueStatus::Failed);
    }

    pub fn cancel(&mut self) {
        self.set_status(SyncQueueStatus::Cancelled);
    }

    pub fn retry(&mut self) {
        self.error_message = None;
        self.set_status(SyncQueueStatus::Pending);
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.get_status(),
            Ok(SyncQueueStatus::Pending) | Ok(SyncQueueStatus::InProgress)
        )
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.get_status(), Ok(SyncQueueStatus::Completed))
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.get_status(), Ok(SyncQueueStatus::Failed))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "upload_queue")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct UploadQueueItem {
    #[ormlite(primary_key)]
    pub id: String,
    pub priority: i32,
    pub cloud_map_id: String,
    pub provider_type: String,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub attempts: i32,
}

impl UploadQueueItem {
    pub fn new(cloud_map_id: String, provider_type: String, priority: i32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            priority,
            cloud_map_id,
            provider_type,
            status: SyncQueueStatus::Pending.as_str().to_string(),
            error_message: None,
            created_at: now,
            updated_at: now,
            attempts: 0,
        }
    }

    pub fn get_status(&self) -> AnyResult<SyncQueueStatus> {
        SyncQueueStatus::from_str(&self.status)
    }

    pub fn set_status(&mut self, status: SyncQueueStatus) {
        self.status = status.as_str().to_string();
        self.updated_at = Utc::now();
    }
    

    pub fn start_processing(&mut self) {
        self.set_status(SyncQueueStatus::InProgress);
    }

    pub fn complete(&mut self) {
        self.set_status(SyncQueueStatus::Completed);
    }

    pub fn fail(&mut self, error: String) {
        self.error_message = Some(error);
        self.attempts += 1;
        self.set_status(SyncQueueStatus::Failed);
    }

    pub fn cancel(&mut self) {
        self.set_status(SyncQueueStatus::Cancelled);
    }

    pub fn retry(&mut self) {
        self.error_message = None;
        self.set_status(SyncQueueStatus::Pending);
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.get_status(),
            Ok(SyncQueueStatus::Pending) | Ok(SyncQueueStatus::InProgress)
        )
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.get_status(), Ok(SyncQueueStatus::Completed))
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.get_status(), Ok(SyncQueueStatus::Failed))
    }
} 