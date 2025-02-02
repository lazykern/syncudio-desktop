mod database;
mod provider;
mod sync;
mod sync_queue;
mod cleanup;
mod metadata;

use chrono::{DateTime, Utc};
use ormlite::Model;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use std::collections::HashMap;
use std::fs::File;
use std::time::UNIX_EPOCH;
use tauri::State;
use uuid::Uuid;

pub use database::*;
pub use provider::*;
pub use sync::*;
pub use sync_queue::*;
pub use cleanup::*;
pub use metadata::*;

use crate::libs::constants::SUPPORTED_TRACKS_EXTENSIONS;
use crate::libs::error::SyncudioError;
use crate::libs::utils::normalize_relative_path;
use crate::plugins::cloud::CloudFile;
use crate::{libs::error::AnyResult, plugins::db::DBState};

use super::models::*;
use super::{CloudProvider, CloudProviderType, CloudState};

use crate::libs::track::Track;
use log::info;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudFolderScanResult {
    /// Number of tracks found in cloud storage
    pub cloud_tracks_found: usize,
    /// Number of local tracks found in the folder
    pub local_tracks_found: usize,
    /// Number of tracks that were newly created in cloud_tracks table
    pub tracks_created: usize,
    /// Number of tracks that were updated with new information
    pub tracks_updated: usize,
    /// Number of track mappings that were cleared (cloud_file_id set to None)
    pub mappings_cleared: usize,
}

#[tauri::command]
pub async fn scan_cloud_music_folder(
    folder_id: String,
    db_state: State<'_, DBState>,
    cloud_state: State<'_, CloudState>,
) -> AnyResult<CloudFolderScanResult> {
    let mut db = db_state.get_lock().await;
    let folder = CloudMusicFolder::select()
        .where_("id = ?")
        .bind(&folder_id)
        .fetch_one(&mut db.connection)
        .await?;

    // Get cloud files for this folder
    let provider = &cloud_state.dropbox;
    let cloud_files = provider
        .list_files(&folder.cloud_folder_id, &folder.cloud_folder_path, true)
        .await?;

    let mut result = CloudFolderScanResult {
        cloud_tracks_found: 0,
        local_tracks_found: 0,
        tracks_created: 0,
        tracks_updated: 0,
        mappings_cleared: 0,
    };

    // Create maps for efficient lookups
    let cloud_files_map: HashMap<String, CloudFile> = cloud_files
        .into_iter()
        .filter(|f| {
            if f.is_folder {
                return false;
            }
            // Check if file has a supported extension
            if let Some(ext) = Path::new(&f.name).extension() {
                if let Some(ext_str) = ext.to_str() {
                    result.cloud_tracks_found += 1;
                    return SUPPORTED_TRACKS_EXTENSIONS.contains(&ext_str.to_lowercase().as_str());
                }
            }
            false
        })
        .map(|f| (f.relative_path.clone(), f))
        .collect();

    // Get local tracks
    let local_tracks = Track::select()
        .where_("path LIKE ?")
        .bind(format!("{}%", folder.local_folder_path))
        .fetch_all(&mut db.connection)
        .await?;

    result.local_tracks_found = local_tracks.len();

    let local_tracks_map: HashMap<String, Track> = local_tracks
        .into_iter()
        .map(|t| {
            let rel_path = t
                .path
                .strip_prefix(&folder.local_folder_path)
                .unwrap_or(&t.path)
                .trim_start_matches('/')
                .to_string();
            (rel_path, t)
        })
        .collect();

    // Get existing cloud tracks and maps for this folder
    let existing_tracks: Vec<(String, String, Option<String>)> = ormlite::query_as(
        "SELECT ct.id, ctm.relative_path, ctm.cloud_file_id 
         FROM cloud_tracks ct 
         INNER JOIN cloud_maps ctm ON ct.id = ctm.cloud_track_id
         WHERE ctm.cloud_music_folder_id = ?"
    )
    .bind(&folder_id)
    .fetch_all(&mut db.connection)
    .await?;

    // Create lookup maps for existing tracks
    let mut existing_by_path: HashMap<String, String> = HashMap::new();
    let mut existing_by_cloud_id: HashMap<String, String> = HashMap::new();
    for (id, path, cloud_id) in existing_tracks {
        existing_by_path.insert(path, id.clone());
        if let Some(cloud_id) = cloud_id {
            existing_by_cloud_id.insert(cloud_id, id);
        }
    }

    let mut processed_track_ids = Vec::new();

    // Process local tracks first
    for (rel_path, local_track) in local_tracks_map.iter() {
        let cloud_file = cloud_files_map.get(rel_path);

        // Try to find existing track ID by path or cloud_file_id
        let existing_id = existing_by_path
            .get(rel_path)
            .cloned()
            .or_else(|| cloud_file.and_then(|f| existing_by_cloud_id.get(&f.id).cloned()));

        match existing_id {
            Some(id) => {
                // Track exists, update if needed
                let mut track = CloudTrack::select()
                    .where_("id = ?")
                    .bind(&id)
                    .fetch_one(&mut db.connection)
                    .await?;

                // Update tags if local file is newer
                let file = File::open(&local_track.path)?;
                let metadata = file.metadata()?;
                let local_mtime = metadata.modified()?.duration_since(UNIX_EPOCH)?;
                let local_updated_at =
                    DateTime::from_timestamp(local_mtime.as_secs() as i64, 0)
                        .unwrap_or_default();

                if local_updated_at > track.updated_at {
                    track.tags = Some(CloudTrackTag::from_track(local_track.clone()));
                    track.updated_at = local_updated_at;
                    track.update_all_fields(&mut db.connection).await?;
                    result.tracks_updated += 1;
                }

                // Update or create track map
                let track_map = CloudTrackMap::select()
                    .where_("cloud_track_id = ? AND cloud_music_folder_id = ?")
                    .bind(&id)
                    .bind(&folder_id)
                    .fetch_optional(&mut db.connection)
                    .await?;

                match track_map {
                    Some(mut map) => {
                        // Update cloud_file_id if needed
                        let new_cloud_id = cloud_file.map(|f| f.id.clone());
                        if map.cloud_file_id != new_cloud_id {
                            map.cloud_file_id = new_cloud_id;
                            map = map.update_all_fields(&mut db.connection).await?;
                            if map.cloud_file_id.is_none() {
                                result.mappings_cleared += 1;
                            }
                        }
                    }
                    None => {
                        // Create new map
                        let map = CloudTrackMap {
                            id: Uuid::new_v4().to_string(),
                            cloud_track_id: id.clone(),
                            cloud_music_folder_id: folder_id.clone(),
                            relative_path: rel_path.clone(),
                            cloud_file_id: cloud_file.map(|f| f.id.clone()),
                        };
                        map.insert(&mut db.connection).await?;
                    }
                }

                processed_track_ids.push(id);
            }
            None => {
                // Create new track
                let track = CloudTrack {
                    id: Uuid::new_v4().to_string(),
                    file_name: local_track.path.split('/').last().unwrap_or("").to_string(),
                    updated_at: Utc::now(),
                    tags: Some(CloudTrackTag::from_track(local_track.clone())),
                };
                let track_id = track.id.clone();
                track.insert(&mut db.connection).await?;

                // Create map
                let map = CloudTrackMap {
                    id: Uuid::new_v4().to_string(),
                    cloud_track_id: track_id.clone(),
                    cloud_music_folder_id: folder_id.clone(),
                    relative_path: rel_path.clone(),
                    cloud_file_id: cloud_file.map(|f| f.id.clone()),
                };
                map.insert(&mut db.connection).await?;

                processed_track_ids.push(track_id);
                result.tracks_created += 1;
            }
        }
    }

    // Process remaining cloud files
    for (rel_path, cloud_file) in cloud_files_map.iter() {
        if local_tracks_map.contains_key(rel_path) {
            continue; // Already processed with local track
        }

        // Try to find existing track by cloud_file_id
        let existing_id = existing_by_cloud_id.get(&cloud_file.id).cloned();

        match existing_id {
            Some(id) if !processed_track_ids.contains(&id) => {
                // Track exists but wasn't processed with local file
                processed_track_ids.push(id);
            }
            None => {
                // Create new track from cloud file
                let track = CloudTrack {
                    id: Uuid::new_v4().to_string(),
                    file_name: cloud_file.name.clone(),
                    updated_at: cloud_file.modified_at,
                    tags: None,
                };
                let track_id = track.id.clone();
                track.insert(&mut db.connection).await?;

                // Create map
                let map = CloudTrackMap {
                    id: Uuid::new_v4().to_string(),
                    cloud_track_id: track_id.clone(),
                    cloud_music_folder_id: folder_id.clone(),
                    relative_path: rel_path.clone(),
                    cloud_file_id: Some(cloud_file.id.clone()),
                };
                map.insert(&mut db.connection).await?;

                processed_track_ids.push(track_id);
                result.tracks_created += 1;
            }
            _ => {} // Already processed
        }
    }

    Ok(result)
}