mod database;
mod provider;

use std::collections::HashMap;
use std::fs::File;
use std::time::UNIX_EPOCH;

pub use database::*;
use ormlite::Model;
pub use provider::*;
use tauri::State;

use crate::libs::constants::SUPPORTED_TRACKS_EXTENSIONS;
use crate::plugins::cloud::CloudFile;
use crate::{libs::error::AnyResult, plugins::db::DBState};

use super::models::cloud_track::{CloudTrack};
use super::{CloudFolder, CloudState};

use crate::libs::track::Track;
use log::info;

#[tauri::command]
pub async fn discover_cloud_folder_tracks(
    folder: CloudFolder,
    cloud_state: State<'_, CloudState>,
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    let mut db = db_state.get_lock().await;
    // Get all cloud files recursively
    let cloud_files = cloud_list_files(
        folder.provider_type.clone(),
        folder.cloud_folder_id.clone(),
        true,
        cloud_state,
    )
    .await?;

    info!("Found {} cloud files", cloud_files.len());

    // Filter for audio files only
    let unprocessed_cloud_audio_files: Vec<CloudFile> = cloud_files.into_iter().filter(|file| {
        !file.is_folder
            && match &file.name.split('.').last() {
                Some(ext) => SUPPORTED_TRACKS_EXTENSIONS.contains(ext),
                None => false,
            }
    }).map(|mut file| {
        if file.relative_path.is_none() {
            file.relative_path = file.display_path.clone().and_then(|display_path| {
                display_path.strip_prefix(&folder.cloud_folder_path).map(|path| path.to_string())
            });
        }

        file
    }).collect();

    // Get all local tracks that are in the cloud folder
    let unprocessed_local_tracks: Vec<Track> = Track::select()
        .fetch_all(&mut db.connection)
        .await?.into_iter()
        .filter(|track| {
            info!("Checking local track: {}", track.path);
            track.path.starts_with(&folder.local_folder_path)
        }).collect();

    // Create a map of relative paths to cloud files for easier lookup
    let cloud_files_map: HashMap<String, CloudFile> = unprocessed_cloud_audio_files
        .into_iter()
        .filter_map(|file| {
            file.relative_path.clone().map(|path| (path, file))
        })
        .collect();

    // Create a map of relative paths to local tracks for easier lookup
    let local_tracks_map: HashMap<String, Track> = unprocessed_local_tracks
        .into_iter()
        .filter_map(|track| {
            track.path.clone().strip_prefix(&folder.local_folder_path)
                .map(|rel_path| (rel_path.to_string(), track))
        })
        .collect();

    // Get existing cloud tracks for this folder
    let existing_cloud_tracks: Vec<CloudTrack> = CloudTrack::select()
        .fetch_all(&mut db.connection)
        .await?;

    let mut cloud_tracks_to_insert = Vec::new();
    let mut cloud_tracks_to_update = Vec::new();

    // Process local tracks that don't have a cloud file yet
    for (rel_path, local_track) in local_tracks_map.iter() {
        let cloud_file = cloud_files_map.get(rel_path);
        let existing_cloud_track = existing_cloud_tracks.iter()
            .find(|ct| ct.blake3_hash == local_track.blake3_hash);

        match (cloud_file, existing_cloud_track) {
            // Track exists locally but not in cloud - create new CloudTrack
            (None, None) => {
                info!("Found new local track: {}", rel_path);
                let cloud_track = CloudTrack::from_track(local_track.clone())?;
                cloud_tracks_to_insert.push(cloud_track);
            }
            // Track exists in both places - update CloudTrack if needed
            (Some(cloud_file), Some(cloud_track)) => {
                info!("Checking if track needs update: {}", rel_path);
                let mut should_update = false;
                let mut updated_track = cloud_track.clone();

                // Update cloud_file_id if changed
                if updated_track.cloud_file_id != Some(cloud_file.id.clone()) {
                    updated_track.cloud_file_id = Some(cloud_file.id.clone());
                    should_update = true;
                }

                // Update local track data if local version is newer
                if local_track.blake3_hash != updated_track.blake3_hash {
                    let file = File::open(&local_track.path)?;
                    let metadata = file.metadata()?;
                    let local_mtime = metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as i64;
                    
                    if local_mtime > updated_track.updated_at {
                        info!("Local track is newer, updating from local");
                        updated_track.blake3_hash = local_track.blake3_hash.clone();
                        if let Some(old_hash) = &updated_track.blake3_hash {
                            if !updated_track.old_blake3_hashes.contains(old_hash) {
                                updated_track.old_blake3_hashes.push(old_hash.clone());
                            }
                        }
                        updated_track.updated_at = local_mtime;
                        should_update = true;
                    }
                }

                if should_update {
                    info!("Updating track: {}", rel_path);
                    cloud_tracks_to_update.push(updated_track);
                }
            }
            // Track exists locally and in cloud but no CloudTrack - create new
            (Some(cloud_file), None) => {
                info!("Creating CloudTrack for existing track: {}", rel_path);
                let mut cloud_track = CloudTrack::from_track(local_track.clone())?;
                cloud_track.cloud_file_id = Some(cloud_file.id.clone());
                cloud_tracks_to_insert.push(cloud_track);
            }
            _ => {}
        }
    }

    // Process cloud-only files
    for (rel_path, cloud_file) in cloud_files_map.iter() {
        if !local_tracks_map.contains_key(rel_path) {
            let existing_cloud_track = existing_cloud_tracks.iter()
                .find(|ct| ct.cloud_file_id == Some(cloud_file.id.clone()));

            if existing_cloud_track.is_none() {
                info!("Found new cloud-only track: {}", rel_path);
                let cloud_track = CloudTrack::from_cloud_file(cloud_file.clone())?;
                cloud_tracks_to_insert.push(cloud_track);
            }
        }
    }

    // Insert new cloud tracks
    for cloud_track in cloud_tracks_to_insert {
        cloud_track.insert(&mut db.connection).await?;
    }

    // Update existing cloud tracks
    for cloud_track in cloud_tracks_to_update {
        cloud_track.update_all_fields(&mut db.connection).await?;
    }

    Ok(())
}