use ormlite::model::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::{fs::File, path::{Path, PathBuf}, time::UNIX_EPOCH, collections::HashMap};
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
    #[serde(skip)]
    pub id: Option<String>,
    pub blake3_hash: Option<String>,
    pub cloud_file_id: Option<String>,
    #[ormlite(json)]
    pub old_blake3_hashes: Vec<String>,
    pub file_name: String,
    pub updated_at: i64,
    #[ormlite(json)]
    pub tags: Option<CloudTrackTag>,
}

impl CloudTrack {
    pub fn from_track(track: Track) -> AnyResult<Self> {
        let now = chrono::Utc::now().timestamp();
        Ok(Self {
            id: Some(Uuid::new_v4().to_string()),
            blake3_hash: track.blake3_hash.clone(),
            cloud_file_id: None,
            old_blake3_hashes: vec![],
            file_name: track.path.split('/').last().unwrap_or("").to_string(),
            updated_at: now,
            tags: Some(CloudTrackTag::from_track(track)),
        })
    }

    pub fn from_cloud_file(cloud_file: CloudFile) -> AnyResult<Self> {
        Ok(Self {
            id: Some(Uuid::new_v4().to_string()),
            blake3_hash: None,
            cloud_file_id: Some(cloud_file.id),
            old_blake3_hashes: vec![],
            file_name: cloud_file.name,
            updated_at: cloud_file.modified_at,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudTracksMetadata {
    pub version: u32,
    pub last_updated: i64,
    pub tracks: Vec<CloudTrack>,
}

impl CloudTracksMetadata {
    pub fn new(tracks: Vec<CloudTrack>) -> Self {
        Self {
            version: 1,
            last_updated: chrono::Utc::now().timestamp(),
            tracks,
        }
    }

    pub fn merge(&mut self, other: CloudTracksMetadata) {
        // If other version is newer, take all its tracks
        if other.version > self.version {
            self.tracks = other.tracks;
            self.version = other.version;
            self.last_updated = other.last_updated;
            return;
        }

        // If same version, merge by most recent update
        if other.version == self.version {
            let mut track_map: HashMap<(Option<String>, Option<String>), CloudTrack> = self.tracks
                .iter()
                .map(|t| ((t.blake3_hash.clone(), t.cloud_file_id.clone()), t.clone()))
                .collect();

            for other_track in other.tracks {
                let key = (other_track.blake3_hash.clone(), other_track.cloud_file_id.clone());
                if let Some(existing) = track_map.get(&key) {
                    if other_track.updated_at > existing.updated_at {
                        track_map.insert(key, other_track);
                    }
                } else {
                    track_map.insert(key, other_track);
                }
            }

            self.tracks = track_map.into_values().collect();
            self.last_updated = chrono::Utc::now().timestamp();
        }
    }

    pub fn to_json(&self) -> AnyResult<String> {
        serde_json::to_string(self).map_err(|e| SyncudioError::SerializationError(e.to_string()))
    }

    pub fn from_json(json: &str) -> AnyResult<Self> {
        serde_json::from_str(json).map_err(|e| SyncudioError::DeserializationError(e.to_string()))
    }
}
