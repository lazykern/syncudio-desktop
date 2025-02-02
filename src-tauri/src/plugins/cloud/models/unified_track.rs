use chrono::{DateTime, Utc};
use ormlite::FromRow;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct UnifiedTrack {
    // Track identifiers
    pub local_track_id: Option<String>,
    pub cloud_track_id: Option<String>,
    pub cloud_map_id: Option<String>,
    pub cloud_folder_id: Option<String>,

    // Paths and locations
    pub local_path: Option<String>,
    pub cloud_relative_path: Option<String>,
    pub cloud_folder_path: Option<String>,
    pub cloud_local_folder_path: Option<String>,
    pub cloud_provider_type: Option<String>,
    pub cloud_file_id: Option<String>,

    // Core metadata
    pub title: String,
    pub album: String,
    #[ormlite(json)]
    pub artists: Option<Vec<String>>,
    #[ormlite(json)]
    pub genres: Option<Vec<String>>,
    pub year: Option<u32>,
    pub duration: u32,
    pub track_no: Option<u32>,
    pub track_of: Option<u32>,
    pub disk_no: Option<u32>,
    pub disk_of: Option<u32>,

    // Location and sync state
    pub location_type: String,  // 'local', 'cloud', or 'both'
    pub cloud_updated_at: Option<DateTime<Utc>>,
} 