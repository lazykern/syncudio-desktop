use chrono::{DateTime, Utc};
use ormlite::model::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
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
    pub composers: Vec<String>,
    pub album_artists: Vec<String>,
    pub genres: Vec<String>,
    pub date: Option<String>,
    pub year: Option<u32>,
    pub duration: u32,
    pub track_no: Option<u32>,
    pub track_of: Option<u32>,
    pub disk_no: Option<u32>,
    pub disk_of: Option<u32>,
    pub bitrate: Option<u32>,
    pub sampling_rate: Option<u32>,
    pub channels: Option<u32>,
    pub encoder: Option<String>,
}

impl CloudTrackTag {
    pub fn from_track(track: Track) -> Self {
        Self {
            title: track.title,
            album: track.album,
            artists: track.artists,
            composers: track.composers,
            album_artists: track.album_artists,
            genres: track.genres,
            date: track.date,
            year: track.year,
            duration: track.duration,
            track_no: track.track_no,
            track_of: track.track_of,
            disk_no: track.disk_no,
            disk_of: track.disk_of,
            bitrate: track.bitrate.map(|b| b * 1000),
            sampling_rate: track.sampling_rate,
            channels: track.channels,
            encoder: track.encoder,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "cloud_tracks")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudTrack {
    #[ormlite(primary_key)]
    pub id: String,
    pub file_name: String,
    pub size: u32,
    pub updated_at: DateTime<Utc>,
    #[ormlite(json)]
    pub tags: Option<CloudTrackTag>,
}

impl CloudTrack {
    pub fn from_track(track: Track) -> AnyResult<Self> {
        let now = chrono::Utc::now();
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            file_name: track.path.split('/').last().unwrap_or("").to_string(),
            size: track.size,
            updated_at: now,
            tags: Some(CloudTrackTag::from_track(track)),
        })
    }

    pub fn from_cloud_file(cloud_file: CloudFile) -> AnyResult<Self> {
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            file_name: cloud_file.name,
            updated_at: cloud_file.modified_at,
            size: cloud_file.size,
            tags: None,
        })
    }
}
