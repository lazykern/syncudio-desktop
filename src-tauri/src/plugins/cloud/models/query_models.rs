use chrono::{DateTime, Utc};
use ormlite::FromRow;
use serde::{Deserialize, Serialize};

use super::cloud_track::CloudTrackTag;

/// Query model for combined track and map data
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TrackWithMapRow {
    // Track fields
    pub id: String,
    pub blake3_hash: Option<String>,
    pub file_name: String,
    pub updated_at: DateTime<Utc>,
    #[ormlite(json)]
    pub tags: Option<CloudTrackTag>,
    // Map fields
    pub map_id: String,
    pub relative_path: String,
    pub cloud_folder_id: String,
    pub cloud_file_id: Option<String>,
}

/// Query model for queue operations
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QueueOperationRow {
    pub queue_type: String,  // 'upload' or 'download'
    pub status: String,
    pub cloud_track_map_id: String,
}

/// Query model for queue statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QueueStatsRow {
    pub status: String,
    pub count: i32,
} 