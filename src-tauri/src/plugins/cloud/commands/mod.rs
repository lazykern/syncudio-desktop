mod database;
mod provider;
mod sync;
mod sync_queue;

use chrono::DateTime;
use ormlite::Model;
use std::collections::HashMap;
use std::fs::File;
use std::time::UNIX_EPOCH;
use tauri::State;
use uuid::Uuid;

pub use database::*;
pub use provider::*;
pub use sync::*;
pub use sync_queue::*;

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

#[tauri::command]
pub async fn discover_cloud_folder_tracks(
    folder_id: String,
    db_state: State<'_, DBState>,
    cloud_state: State<'_, CloudState>,
) -> AnyResult<()> {
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

    // Get ALL existing cloud tracks that match our hashes or have maps with our cloud_file_ids
    let mut hash_params = Vec::new();
    let mut cloud_id_params = Vec::new();

    for track in local_tracks_map.values() {
        if let Some(hash) = &track.blake3_hash {
            hash_params.push(hash);
        }
    }
    for file in cloud_files_map.values() {
        cloud_id_params.push(&file.id);
    }

    let mut query = String::from(
        "SELECT DISTINCT ct.id, ct.blake3_hash, ctm.cloud_file_id 
         FROM cloud_tracks ct 
         LEFT JOIN cloud_track_maps ctm ON ct.id = ctm.cloud_track_id
         WHERE 0=1",
    );

    if !hash_params.is_empty() {
        query.push_str(" OR ct.blake3_hash IN (");
        query.push_str(
            &std::iter::repeat("?")
                .take(hash_params.len())
                .collect::<Vec<_>>()
                .join(","),
        );
        query.push(')');
    }
    if !cloud_id_params.is_empty() {
        query.push_str(" OR ctm.cloud_file_id IN (");
        query.push_str(
            &std::iter::repeat("?")
                .take(cloud_id_params.len())
                .collect::<Vec<_>>()
                .join(","),
        );
        query.push(')');
    }

    let mut stmt = ormlite::query_as::<_, (String, Option<String>, Option<String>)>(&query);
    for param in hash_params {
        stmt = stmt.bind(param);
    }
    for param in cloud_id_params {
        stmt = stmt.bind(param);
    }

    let existing_tracks: Vec<(String, Option<String>, Option<String>)> = stmt.fetch_all(&mut db.connection).await?;

    // Create lookup maps for existing tracks
    let mut existing_by_hash: HashMap<String, String> = HashMap::new();
    let mut existing_by_cloud_id: HashMap<String, String> = HashMap::new();
    for (id, hash, cloud_id) in existing_tracks {
        if let Some(hash) = hash {
            existing_by_hash.insert(hash, id.clone());
        }
        if let Some(cloud_id) = cloud_id {
            existing_by_cloud_id.insert(cloud_id, id);
        }
    }

    let mut processed_track_ids = Vec::new();

    // Process local tracks first
    for (rel_path, local_track) in local_tracks_map.iter() {
        let cloud_file = cloud_files_map.get(rel_path);

        // Try to find existing track ID, first by hash then by cloud_file_id
        let existing_id = if let Some(hash) = &local_track.blake3_hash {
            // First check if we already have this track in the database
            match existing_by_hash.get(hash).cloned() {
                Some(id) => Some(id),
                None => CloudTrack::select()
                    .where_("blake3_hash = ?")
                    .bind(hash)
                    .fetch_optional(&mut db.connection)
                    .await
                    .ok()
                    .flatten()
                    .map(|t| t.id),
            }
        } else {
            None
        }
        .or_else(|| cloud_file.and_then(|f| existing_by_cloud_id.get(&f.id).cloned()));

        match existing_id {
            Some(id) => {
                // Update existing track
                let mut track = CloudTrack::select()
                    .where_("id = ?")
                    .bind(&id)
                    .fetch_one(&mut db.connection)
                    .await?;

                let mut updated = false;

                // Update hash and tags if local file is newer
                if let Some(hash) = &local_track.blake3_hash {
                    if track.blake3_hash.as_ref() != Some(hash) {
                        let file = File::open(&local_track.path)?;
                        let metadata = file.metadata()?;
                        let local_mtime = metadata.modified()?.duration_since(UNIX_EPOCH)?;
                        let local_updated_at =
                            DateTime::from_timestamp(local_mtime.as_secs() as i64, 0)
                                .unwrap_or_default();

                        if local_updated_at > track.updated_at {
                            track.blake3_hash = Some(hash.clone());
                            track.tags = Some(CloudTrackTag::from_track(local_track.clone()));
                            track.updated_at = local_updated_at;
                            updated = true;
                        }
                    }
                }

                if updated {
                    track.update_all_fields(&mut db.connection).await?;
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
                        let mut map_updated = false;
                        
                        // Update cloud_file_id based on cloud storage state
                        if let Some(cloud_file) = cloud_file {
                            if map.cloud_file_id.as_ref() != Some(&cloud_file.id) {
                                map.cloud_file_id = Some(cloud_file.id.clone());
                                map_updated = true;
                            }
                        } else if map.cloud_file_id.is_some() {
                            // Clear cloud_file_id if file no longer exists in cloud storage
                            map.cloud_file_id = None;
                            map_updated = true;
                            info!("Clearing cloud_file_id for track {} as file no longer exists in cloud storage", id);
                        }

                        if map_updated {
                            map.update_all_fields(&mut db.connection).await?;
                        }
                    }
                    None => {
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
                // Create new track and map
                let track = CloudTrack::from_track(local_track.clone())?;
                let track_id = track.id.clone();
                track.insert(&mut db.connection).await?;

                let map = CloudTrackMap {
                    id: Uuid::new_v4().to_string(),
                    cloud_track_id: track_id.clone(),
                    cloud_music_folder_id: folder_id.clone(),
                    relative_path: rel_path.clone(),
                    cloud_file_id: cloud_file.map(|f| f.id.clone()),
                };
                map.insert(&mut db.connection).await?;

                processed_track_ids.push(track_id);
            }
        }
    }

    // Process cloud-only files
    for (rel_path, cloud_file) in cloud_files_map.iter() {
        if local_tracks_map.contains_key(rel_path) {
            continue; // Already processed with local tracks
        }

        // Try to find existing track
        if let Some(id) = existing_by_cloud_id.get(&cloud_file.id) {
            if !processed_track_ids.contains(id) {
                // Update track map if needed
                let track_map = CloudTrackMap::select()
                    .where_("cloud_track_id = ? AND cloud_music_folder_id = ?")
                    .bind(id)
                    .bind(&folder_id)
                    .fetch_optional(&mut db.connection)
                    .await?;

                match track_map {
                    Some(mut map) => {
                        if map.cloud_file_id.as_ref() != Some(&cloud_file.id) {
                            map.cloud_file_id = Some(cloud_file.id.clone());
                            map.update_all_fields(&mut db.connection).await?;
                        }
                    }
                    None => {
                        let map = CloudTrackMap {
                            id: Uuid::new_v4().to_string(),
                            cloud_track_id: id.clone(),
                            cloud_music_folder_id: folder_id.clone(),
                            relative_path: rel_path.clone(),
                            cloud_file_id: Some(cloud_file.id.clone()),
                        };
                        map.insert(&mut db.connection).await?;
                    }
                }
                processed_track_ids.push(id.clone());
            }
        } else {
            // Create new track and map
            let track = CloudTrack::from_cloud_file(cloud_file.clone())?;
            let track_id = track.id.clone();
            track.insert(&mut db.connection).await?;

            let map = CloudTrackMap {
                id: Uuid::new_v4().to_string(),
                cloud_track_id: track_id.clone(),
                cloud_music_folder_id: folder_id.clone(),
                relative_path: rel_path.clone(),
                cloud_file_id: Some(cloud_file.id.clone()),
            };
            map.insert(&mut db.connection).await?;

            processed_track_ids.push(track_id);
        }
    }

    // Clear cloud_file_id for any maps that reference files no longer in cloud storage
    let mut orphaned_maps = CloudTrackMap::select()
        .where_("cloud_music_folder_id = ? AND cloud_file_id IS NOT NULL")
        .bind(&folder_id)
        .fetch_all(&mut db.connection)
        .await?;

    for mut map in orphaned_maps {
        if let Some(cloud_file_id) = &map.cloud_file_id {
            if !cloud_files_map.values().any(|f| f.id == *cloud_file_id) {
                info!("Clearing cloud_file_id for orphaned map {} as file no longer exists in cloud storage", map.id);
                map.cloud_file_id = None;
                map.update_all_fields(&mut db.connection).await?;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn sync_cloud_tracks_metadata(
    provider_type: String,
    cloud_state: State<'_, CloudState>,
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    let mut db = db_state.get_lock().await;
    let provider = CloudProviderType::from_str(&provider_type)?;

    // Get local cloud tracks
    let local_tracks = CloudTrack::select().fetch_all(&mut db.connection).await?;
    let local_metadata = CloudTracksMetadata::new(local_tracks);

    // Download cloud metadata if exists
    match provider {
        CloudProviderType::Dropbox => {
            match cloud_state.dropbox.download_metadata().await? {
                Some(mut cloud_metadata) => {
                    // Merge cloud metadata with local
                    cloud_metadata.merge(local_metadata);

                    // Update local database with merged tracks
                    for track in &cloud_metadata.tracks {
                        match CloudTrack::select()
                            .where_("blake3_hash = ? OR cloud_file_id = ? OR id = ?")
                            .bind(&track.blake3_hash)
                            .bind(&track.id)
                            .fetch_optional(&mut db.connection)
                            .await?
                        {
                            Some(existing) => {
                                info!("Updating existing track: {:?}", existing);
                                existing.update_all_fields(&mut db.connection).await?;
                            }
                            None => {
                                info!("Inserting new track: {:?}", track);
                                (*track).clone().insert(&mut db.connection).await?;
                            }
                        }
                    }

                    // Upload merged metadata back to cloud
                    cloud_state.dropbox.upload_metadata(&cloud_metadata).await?;
                }
                None => {
                    // No cloud metadata exists, upload local
                    cloud_state.dropbox.upload_metadata(&local_metadata).await?;
                }
            }
        }
        CloudProviderType::GoogleDrive => {
            return Err(SyncudioError::GoogleDrive(
                "Google Drive not implemented yet".to_string(),
            ))
        }
    }

    Ok(())
}
