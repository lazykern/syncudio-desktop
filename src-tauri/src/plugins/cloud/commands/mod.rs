mod database;
mod provider;
mod sync;

use chrono::DateTime;
use std::collections::HashMap;
use std::fs::File;
use std::time::UNIX_EPOCH;
use uuid::Uuid;

pub use database::*;
use ormlite::Model;
pub use provider::*;
pub use sync::*;
use tauri::State;

use crate::libs::constants::SUPPORTED_TRACKS_EXTENSIONS;
use crate::libs::error::SyncudioError;
use crate::libs::utils::normalize_relative_path;
use crate::plugins::cloud::CloudFile;
use crate::{libs::error::AnyResult, plugins::db::DBState};

use super::cloud_folder::CloudFolder;
use super::cloud_track::{CloudTrackTag, CloudTracksMetadata};
use super::models::cloud_track::{CloudTrack, CloudTrackMap};
use super::{CloudProvider, CloudProviderType, CloudState};

use crate::libs::track::Track;
use log::info;

#[tauri::command]
pub async fn discover_cloud_folder_tracks(
    folder_id: String,
    db_state: State<'_, DBState>,
    cloud_state: State<'_, CloudState>,
) -> AnyResult<()> {
    let mut db = db_state.get_lock().await;
    let folder = CloudFolder::select()
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
        .filter(|f| !f.is_folder)
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

    // Get ALL existing cloud tracks that match our hashes or cloud_file_ids
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
        "SELECT DISTINCT ct.id, ct.blake3_hash, ct.cloud_file_id 
         FROM cloud_tracks ct 
         WHERE 0=1",
    );

    if !hash_params.is_empty() {
        query.push_str(" OR blake3_hash IN (");
        query.push_str(
            &std::iter::repeat("?")
                .take(hash_params.len())
                .collect::<Vec<_>>()
                .join(","),
        );
        query.push(')');
    }
    if !cloud_id_params.is_empty() {
        query.push_str(" OR cloud_file_id IN (");
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

    let existing_tracks = stmt.fetch_all(&mut db.connection).await?;

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

    let mut tracks_to_process = Vec::new();
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

                // Update cloud_file_id if needed
                if let Some(cloud_file) = cloud_file {
                    if track.cloud_file_id.as_ref() != Some(&cloud_file.id) {
                        track.cloud_file_id = Some(cloud_file.id.clone());
                        updated = true;
                    }
                }

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
                            if let Some(old_hash) = track.blake3_hash.take() {
                                if !track.old_blake3_hashes.contains(&old_hash) {
                                    track.old_blake3_hashes.push(old_hash);
                                }
                            }
                            track.blake3_hash = Some(hash.clone());
                            track.tags = Some(CloudTrackTag::from_track(local_track.clone()));
                            track.updated_at = local_updated_at;
                            updated = true;
                        }
                    }
                }

                if updated {
                    tracks_to_process.push((track, None));
                }
                processed_track_ids.push(id);
            }
            None => {
                // Create new track
                let mut track = CloudTrack::from_track(local_track.clone())?;
                if let Some(cloud_file) = cloud_file {
                    track.cloud_file_id = Some(cloud_file.id.clone());
                }
                tracks_to_process.push((track, Some(rel_path.clone())));
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
                let track = CloudTrack::select()
                    .where_("id = ?")
                    .bind(id)
                    .fetch_one(&mut db.connection)
                    .await?;
                tracks_to_process.push((track, None));
                processed_track_ids.push(id.clone());
            }
        } else {
            let track = CloudTrack::from_cloud_file(cloud_file.clone())?;
            tracks_to_process.push((track, Some(rel_path.clone())));
        }
    }

    // Process all tracks in batches
    for (track, maybe_rel_path) in tracks_to_process {
        // Update or insert track
        let track_id = if maybe_rel_path.is_none() {
            track.update_all_fields(&mut db.connection).await?.id
        } else {
            // For new tracks, first check if a track with this hash already exists
            if let Some(hash) = &track.blake3_hash {
                if let Some(existing) = CloudTrack::select()
                    .where_("blake3_hash = ?")
                    .bind(hash)
                    .fetch_optional(&mut db.connection)
                    .await?
                {
                    // Update existing track if needed
                    let mut existing = existing;
                    if existing.cloud_file_id.is_none() && track.cloud_file_id.is_some() {
                        existing.cloud_file_id = track.cloud_file_id;
                        existing = existing.update_all_fields(&mut db.connection).await?;
                    }
                    existing.id
                } else {
                    track.insert(&mut db.connection).await?.id
                }
            } else {
                track.insert(&mut db.connection).await?.id
            }
        };

        // Create or update track map if we have a relative path
        if let Some(rel_path) = maybe_rel_path {
            // Check for existing map
            let existing_map = CloudTrackMap::select()
                .where_("cloud_track_id = ? AND cloud_folder_id = ?")
                .bind(&track_id)
                .bind(&folder_id)
                .fetch_optional(&mut db.connection)
                .await?;

            match existing_map {
                Some(mut map) => {
                    if map.relative_path != rel_path {
                        map.relative_path = rel_path;
                        map.update_all_fields(&mut db.connection).await?;
                    }
                }
                None => {
                    let map = CloudTrackMap {
                        id: Uuid::new_v4().to_string(),
                        cloud_track_id: track_id,
                        cloud_folder_id: folder_id.clone(),
                        relative_path: rel_path,
                    };
                    map.insert(&mut db.connection).await?;
                }
            }
        }
    }

    // Clean up orphaned tracks (no cloud_file_id and no blake3_hash)
    ormlite::query(
        r#"
        DELETE FROM cloud_tracks 
        WHERE blake3_hash IS NULL 
        AND cloud_file_id IS NULL
        "#,
    )
    .execute(&mut db.connection)
    .await?;

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
                            .bind(&track.cloud_file_id)
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
