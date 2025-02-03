use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::CloudTrackTag;

/// Represents track metadata stored in cloud storage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudTrackMetadata {
    // Core identifiers
    pub cloud_file_id: String,

    // Paths
    pub cloud_path: String,          // Absolute path in cloud storage
    pub relative_path: String,       // Path relative to cloud folder

    // Track metadata (reusing existing tag structure)
    pub tags: Option<CloudTrackTag>,

    // Cloud sync metadata
    #[serde(rename = "last_modified")]
    pub last_modified: String,       // Timestamp in milliseconds as string
    #[serde(rename = "last_sync")]
    pub last_sync: Option<DateTime<Utc>>,
    pub provider: String,            // e.g. "dropbox"
    pub cloud_folder_id: String,     // Reference to parent cloud folder
}

/// Collection of track metadata for cloud storage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudMetadataCollection {
    pub tracks: Vec<CloudTrackMetadata>,
}

impl CloudMetadataCollection {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
        }
    }
}

/// Result of a metadata sync operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudMetadataSyncResult {
    pub tracks_updated: u32,      // Number of tracks updated from cloud
    pub tracks_created: u32,      // Number of new tracks created from cloud
    pub is_fresh_start: bool,     // Whether this was the first sync
}

impl CloudMetadataSyncResult {
    pub fn new(is_fresh_start: bool) -> Self {
        Self {
            tracks_updated: 0,
            tracks_created: 0,
            is_fresh_start,
        }
    }
}

/// Result of a metadata update operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudMetadataUpdateResult {
    pub tracks_included: u32,     // Number of tracks included in metadata
    pub tracks_skipped: u32,      // Number of tracks skipped (missing cloud_id)
}

impl CloudMetadataUpdateResult {
    pub fn new() -> Self {
        Self {
            tracks_included: 0,
            tracks_skipped: 0,
        }
    }
} 