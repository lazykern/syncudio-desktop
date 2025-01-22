use chrono::{DateTime, Utc};
use ormlite::FromRow;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::cloud_track::CloudTrackTag;

/// Represents the location state of a track by checking both local and cloud existence by blake3_hash, cloud_file_id and relative_path (should be in local storage and cloud storage)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
#[serde(rename_all = "snake_case")]
pub enum TrackLocationState {
    /// Track exists in both local and cloud, and is in sync
    Complete,
    /// Track exists only in local storage
    LocalOnly,
    /// Track exists only in cloud storage
    CloudOnly,
    /// Track exists in both but has different hashes
    OutOfSync,
    /// Track is mapped but missing from both locations
    Missing,
    /// Track exists but has no mapping
    NotMapped,
}

/// Represents operation type for sync operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
#[serde(rename_all = "snake_case")]
pub enum SyncOperationType {
    Upload,
    Download,
}

/// Represents the status of a sync operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Pending,
    InProgress,
    Completed,
    Failed {
        error: String,
        attempts: i32,
    },
}

/// Represents the sync status of a cloud folder
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
#[serde(rename_all = "snake_case")]
pub enum FolderSyncStatus {
    /// All tracks in the folder are synced
    Synced,
    /// Some tracks are being synced
    Syncing,
    /// Some tracks need attention (out of sync, missing, etc)
    NeedsAttention,
    /// Folder has no tracks
    Empty,
}

/// Represents a track with its current sync and integrity status
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudTrackDTO {
    pub id: String,
    pub cloud_music_folder_id: String,
    pub cloud_track_map_id: String,
    pub file_name: String,
    pub relative_path: String,
    pub location_state: TrackLocationState,
    pub sync_operation: Option<SyncOperationType>,
    pub sync_status: Option<SyncStatus>,
    pub updated_at: DateTime<Utc>,
    pub tags: Option<CloudTrackTag>,
}

/// Represents a sync queue item
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct QueueItemDTO {
    pub id: String,
    pub cloud_track_id: String,
    pub file_name: String,
    pub operation: SyncOperationType,
    pub status: SyncStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub provider_type: String,
}

/// Represents queue statistics
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct QueueStatsDTO {
    pub pending_count: i32,
    pub in_progress_count: i32,
    pub completed_count: i32,
    pub failed_count: i32,
}

/// Represents a sync history entry
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct SyncHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub operation: SyncOperationType,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    pub status: SyncStatus,
}

/// Represents detailed sync information for a track
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct TrackSyncStatusDTO {
    pub location_state: TrackLocationState,
    pub sync_operation: Option<SyncOperationType>,
    pub sync_status: Option<SyncStatus>,
    pub updated_at: DateTime<Utc>,
}

/// Represents detailed sync information for a cloud folder
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudFolderSyncDetailsDTO {
    pub id: String,
    pub cloud_folder_path: String,
    pub local_folder_path: String,
    pub sync_status: FolderSyncStatus,
    pub pending_sync_count: i32,
    pub tracks: Vec<CloudTrackDTO>,
}

/// DTO for queue statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QueueStatsGroupDTO {
    pub status: String,
    pub count: i32,
}