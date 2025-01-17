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
#[ormlite(table = "cloud_track_paths")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudTrackPath {
    #[ormlite(primary_key)]
    pub id: Option<String>,
    pub cloud_track_id: String,
    pub cloud_folder_id: String,
    pub relative_path: String,
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

    pub fn needs_update_from_cloud_file(&self, cloud_file: &CloudFile) -> bool {
        self.cloud_file_id != Some(cloud_file.id.clone())
    }

    pub fn needs_update_from_local_track(&self, local_track: &Track, local_mtime: i64) -> bool {
        local_track.blake3_hash != self.blake3_hash && local_mtime > self.updated_at
    }

    pub fn update_from_cloud_file(&mut self, cloud_file: &CloudFile) {
        self.cloud_file_id = Some(cloud_file.id.clone());
    }

    pub fn update_from_local_track(&mut self, local_track: &Track, local_mtime: i64) {
        if let Some(old_hash) = &self.blake3_hash {
            if !self.old_blake3_hashes.contains(old_hash) {
                self.old_blake3_hashes.push(old_hash.clone());
            }
        }
        self.blake3_hash = local_track.blake3_hash.clone();
        self.updated_at = local_mtime;
    }

    pub fn matches_track(&self, cloud_file: Option<&CloudFile>, local_track: &Track) -> bool {
        (cloud_file.is_some() && self.cloud_file_id.as_ref() == cloud_file.map(|f| &f.id)) ||
        (self.blake3_hash == local_track.blake3_hash)
    }

    pub fn matches_cloud_file(&self, cloud_file: &CloudFile, local_track: Option<&Track>) -> bool {
        (self.cloud_file_id.as_ref() == Some(&cloud_file.id)) ||
        (local_track.is_some() && self.blake3_hash.as_ref() == local_track.and_then(|t| t.blake3_hash.as_ref()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudTracksMetadata {
    pub tracks: Vec<CloudTrack>,
}

impl CloudTracksMetadata {
    pub fn new(tracks: Vec<CloudTrack>) -> Self {
        Self { tracks }
    }

    pub fn merge(&mut self, other: CloudTracksMetadata) {
        // Create a map of all tracks, keeping the most recent version of each
        let mut track_map: HashMap<(Option<String>, Option<String>), CloudTrack> = self.tracks
            .iter()
            .map(|t| ((t.blake3_hash.clone(), t.cloud_file_id.clone()), t.clone()))
            .collect();

        // Merge in other tracks, keeping the most recent version
        for other_track in other.tracks {
            let key = (other_track.blake3_hash.clone(), other_track.cloud_file_id.clone());
            match track_map.get(&key) {
                Some(existing) if other_track.updated_at > existing.updated_at => {
                    track_map.insert(key, other_track);
                }
                None => {
                    track_map.insert(key, other_track);
                }
                _ => {}
            }
        }

        self.tracks = track_map.into_values().collect();
    }

    pub fn to_json(&self) -> AnyResult<String> {
        serde_json::to_string(self).map_err(|e| SyncudioError::SerializationError(e.to_string()))
    }

    pub fn from_json(json: &str) -> AnyResult<Self> {
        serde_json::from_str(json).map_err(|e| SyncudioError::DeserializationError(e.to_string()))
    }
}
