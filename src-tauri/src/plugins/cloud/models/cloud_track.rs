use ormlite::model::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;

use crate::libs::{
    error::{AnyResult, SyncudioError},
    track::Track,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "cloud_tracks")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudTrack {
    #[ormlite(primary_key)]
    pub blake3_hash: String,
    pub cloud_file_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub file_name: String,
    pub title: String,
    pub album: String,
    #[ormlite(json)]
    pub artists: Vec<String>,
    #[ormlite(json)]
    pub genres: Vec<String>,
    pub year: Option<u32>,
    pub duration: u32,
    pub track_no: Option<u32>,
    pub track_of: Option<u32>,
    pub disk_no: Option<u32>,
    pub disk_of: Option<u32>,
}

impl CloudTrack {
    pub fn from_track(track: Track, blake3_hash: String) -> AnyResult<Self> {
        let now = chrono::Utc::now().timestamp();
        Ok(Self {
            blake3_hash,
            cloud_file_id: None,
            created_at: now,
            updated_at: now,
            file_name: PathBuf::from(&track.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
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
        })
    }

    pub fn set_cloud_file_id(&mut self, cloud_file_id: String) {
        self.cloud_file_id = Some(cloud_file_id);
    }
}
