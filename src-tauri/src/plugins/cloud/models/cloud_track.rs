use ormlite::model::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;
use ts_rs::TS;

use crate::{libs::{
    error::{AnyResult, SyncudioError},
    track::Track,
}, plugins::cloud::CloudFile};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudTrackTag {
    pub title: String,
    pub album: String,
    pub artists: Vec<String>,
    pub genres: Vec<String>,
    pub year: Option<u32>,
    pub duration: u32,
    pub track_no: Option<u32>,
    pub track_of: Option<u32>,
    pub disk_no: Option<u32>,
    pub disk_of: Option<u32>,
}

impl CloudTrackTag {
    pub fn from_track(track: Track) -> Self {
        Self {
            title: track.title,
            album: track.album,
            artists: track.artists,
            genres: track.genres,
            year: track.year,
            duration: track.duration,
            track_no: track.track_no,
            track_of: track.track_of,
            disk_no: track.disk_no,
            disk_of: track.disk_of,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "cloud_tracks")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudTrack {
    #[ormlite(primary_key)]
    pub id: String,
    pub blake3_hash: Option<String>,
    #[ormlite(json)]
    pub old_blake3_hash: Vec<String>,
    pub cloud_file_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub file_name: String,
    #[ormlite(json)]
    pub tags: Option<CloudTrackTag>,
}

impl CloudTrack {
    pub fn from_track(track: Track) -> AnyResult<Self> {
        let now = chrono::Utc::now().timestamp();
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            blake3_hash: track.blake3_hash.clone(),
            old_blake3_hash: vec![],
            cloud_file_id: None,
            created_at: now,
            updated_at: now,
            file_name: PathBuf::from(&track.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            tags: Some(CloudTrackTag::from_track(track)),
        })
    }

    pub fn from_cloud_file(cloud_file: CloudFile) -> AnyResult<Self> {
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            blake3_hash: None,
            old_blake3_hash: vec![],
            cloud_file_id: Some(cloud_file.id),
            created_at: cloud_file.created_at,
            updated_at: cloud_file.modified_at,
            file_name: cloud_file.name,
            tags: None,
        })
    }

    pub fn set_blake3_hash(&mut self, blake3_hash: String) {
        self.blake3_hash = Some(blake3_hash);
    }

    pub fn set_cloud_file_id(&mut self, cloud_file_id: String) {
        self.cloud_file_id = Some(cloud_file_id);
    }

    pub fn set_tags(&mut self, tags: CloudTrackTag) {
        self.tags = Some(tags);
    }
}
