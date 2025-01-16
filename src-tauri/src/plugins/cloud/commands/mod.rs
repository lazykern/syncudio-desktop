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
use crate::libs::error::SyncudioError;
use crate::plugins::cloud::CloudFile;
use crate::{libs::error::AnyResult, plugins::db::DBState};

use super::models::cloud_track::{CloudTrack};
use super::{CloudFolder, CloudProvider, CloudProviderType, CloudState, CloudTracksMetadata};

use crate::libs::track::Track;
use log::info;

fn filter_audio_files(files: Vec<CloudFile>, folder_path: &str) -> Vec<CloudFile> {
    files.into_iter()
        .filter(|file| {
            !file.is_folder && match &file.name.split('.').last() {
                Some(ext) => SUPPORTED_TRACKS_EXTENSIONS.contains(ext),
                None => false,
            }
        })
        .map(|mut file| {
            if file.relative_path.is_none() {
                file.relative_path = file.display_path.clone().and_then(|display_path| {
                    display_path.strip_prefix(folder_path).map(|path| path.to_string())
                });
            }
            file
        })
        .collect()
}

fn create_path_maps(
    cloud_files: Vec<CloudFile>,
    local_tracks: Vec<Track>,
    folder: &CloudFolder,
) -> (HashMap<String, CloudFile>, HashMap<String, Track>) {
    let cloud_files_map: HashMap<String, CloudFile> = cloud_files
        .into_iter()
        .filter_map(|file| {
            file.relative_path.clone().map(|path| (path, file))
        })
        .collect();

    let local_tracks_map: HashMap<String, Track> = local_tracks
        .into_iter()
        .filter_map(|track| {
            track.path.clone().strip_prefix(&folder.local_folder_path)
                .map(|rel_path| (rel_path.to_string(), track))
        })
        .collect();

    (cloud_files_map, local_tracks_map)
}

fn process_track_updates(
    local_track: &Track,
    cloud_file: Option<&CloudFile>,
    cloud_track: Option<&CloudTrack>,
    rel_path: &str,
) -> Option<CloudTrack> {
    match (cloud_file, cloud_track) {
        // Track exists locally but not in cloud - CloudTrack not created yet -> create
        (None, None) => {
            info!("Found new local track: {}", rel_path);
            CloudTrack::from_track(local_track.clone()).ok()
        }
        // Track exists in both places - CloudTrack created -> update if needed
        (Some(cloud_file), Some(cloud_track)) => {
            info!("Checking if track needs update: {}", rel_path);
            let mut should_update = false;
            let mut updated_track = cloud_track.clone();

            // Update cloud_file_id if changed
            if updated_track.needs_update_from_cloud_file(cloud_file) {
                updated_track.update_from_cloud_file(cloud_file);
                should_update = true;
            }

            // Update local track data if local version is newer
            if let Ok(file) = File::open(&local_track.path) {
                if let Ok(metadata) = file.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                            let local_mtime = duration.as_secs() as i64;
                            if updated_track.needs_update_from_local_track(local_track, local_mtime) {
                                updated_track.update_from_local_track(local_track, local_mtime);
                                should_update = true;
                            }
                        }
                    }
                }
            }

            if should_update {
                info!("Updating track: {}", rel_path);
                Some(updated_track)
            } else {
                None
            }
        }
        // Track exists both locally and in cloud - CloudTrack not created yet -> create
        (Some(cloud_file), None) => {
            info!("Creating CloudTrack for existing track: {}", rel_path);
            let mut cloud_track = CloudTrack::from_track(local_track.clone()).ok()?;
            cloud_track.cloud_file_id = Some(cloud_file.id.clone());
            Some(cloud_track)
        }
        // Tracks exists locally but not in cloud - CloudTrack created -> keep existing
        (None, Some(_)) => None,
    }
}

fn process_cloud_file_updates(
    cloud_file: &CloudFile,
    local_track: Option<&Track>,
    cloud_track: Option<&CloudTrack>,
    rel_path: &str,
) -> Option<CloudTrack> {
    match (local_track, cloud_track) {
        // Track exists in cloud but not locally - CloudTrack not created yet -> create
        (None, None) => {
            info!("Found new cloud-only track: {}", rel_path);
            CloudTrack::from_cloud_file(cloud_file.clone()).ok()
        }
        // Track exists in both places - CloudTrack created -> update if needed
        (Some(local_track), Some(cloud_track)) => {
            info!("Checking if track needs update: {}", rel_path);
            let mut updated_track = cloud_track.clone();

            // Update local track data if local version is newer
            if local_track.blake3_hash != updated_track.blake3_hash {
                updated_track.blake3_hash = local_track.blake3_hash.clone();
                Some(updated_track)
            } else {
                None
            }
        }
        // Tracks exists in both places - CloudTrack not created yet -> create
        (Some(local_track), None) => {
            info!("Creating CloudTrack for existing track: {}", rel_path);
            let mut cloud_track = CloudTrack::from_track(local_track.clone()).ok()?;
            cloud_track.cloud_file_id = Some(cloud_file.id.clone());
            Some(cloud_track)
        }
        // Track does not exist locally but exists in cloud - CloudTrack created -> keep existing
        (None, Some(_)) => None,
    }
}

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
    ).await?;

    info!("Found {} cloud files", cloud_files.len());

    // Filter for audio files only
    let cloud_audio_files = filter_audio_files(cloud_files, &folder.cloud_folder_path);

    // Get all local tracks that are in the cloud folder
    let local_tracks: Vec<Track> = Track::select()
        .fetch_all(&mut db.connection)
        .await?
        .into_iter()
        .filter(|track| track.path.starts_with(&folder.local_folder_path))
        .collect();

    // Create lookup maps
    let (cloud_files_map, local_tracks_map) = create_path_maps(
        cloud_audio_files,
        local_tracks,
        &folder
    );

    // Get existing cloud tracks
    let existing_cloud_tracks: Vec<CloudTrack> = CloudTrack::select()
        .fetch_all(&mut db.connection)
        .await?;

    let mut tracks_to_save: HashMap<(Option<String>, Option<String>), CloudTrack> = HashMap::new();

    // Process local tracks
    for (rel_path, local_track) in local_tracks_map.iter() {
        let cloud_file = cloud_files_map.get(rel_path);
        let existing_cloud_track = existing_cloud_tracks.iter()
            .find(|ct| ct.matches_track(cloud_file, local_track));

        if let Some(track) = process_track_updates(
            local_track,
            cloud_file,
            existing_cloud_track,
            rel_path,
        ) {
            let key = (track.blake3_hash.clone(), track.cloud_file_id.clone());
            if !tracks_to_save.contains_key(&key) || 
               tracks_to_save.get(&key).map_or(true, |existing| track.updated_at > existing.updated_at) {
                tracks_to_save.insert(key, track);
            }
        }
    }

    // Process cloud files
    for (rel_path, cloud_file) in cloud_files_map.iter() {
        let local_track = local_tracks_map.get(rel_path);
        let existing_cloud_track = existing_cloud_tracks.iter()
            .find(|ct| ct.matches_cloud_file(cloud_file, local_track));

        if let Some(track) = process_cloud_file_updates(
            cloud_file,
            local_track,
            existing_cloud_track,
            rel_path,
        ) {
            let key = (track.blake3_hash.clone(), track.cloud_file_id.clone());
            if !tracks_to_save.contains_key(&key) || 
               tracks_to_save.get(&key).map_or(true, |existing| track.updated_at > existing.updated_at) {
                tracks_to_save.insert(key, track);
            }
        }
    }

    // Save tracks
    for track in tracks_to_save.values() {
        info!("Processing cloud track: ({:?}, {:?})", track.cloud_file_id, track.blake3_hash);
        match existing_cloud_tracks.iter().find(|ct| 
            (ct.cloud_file_id.is_some() && ct.cloud_file_id == track.cloud_file_id) ||
            (ct.blake3_hash.is_some() && ct.blake3_hash == track.blake3_hash)
        ) {
            Some(_) => {
                info!("Updating existing track");
                track.clone().update_all_fields(&mut db.connection).await?;
            }
            None => {
                info!("Inserting new track");
                track.clone().insert(&mut db.connection).await?;
            }
        }
    }

    // Clean up orphaned tracks
    CloudTrack::query(r#"
        DELETE FROM cloud_tracks WHERE blake3_hash IS NULL AND cloud_file_id IS NULL
    "#).fetch_all(&mut db.connection).await?;

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
    let local_tracks = CloudTrack::select()
        .fetch_all(&mut db.connection)
        .await?;
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
                            .where_("(blake3_hash = ? OR blake3_hash IS NULL) AND (cloud_file_id = ? OR cloud_file_id IS NULL)")
                            .bind(&track.blake3_hash)
                            .bind(&track.cloud_file_id)
                            .fetch_optional(&mut db.connection)
                            .await? {
                                Some(existing) => {
                                    existing.update_all_fields(&mut db.connection).await?;
                                }
                                None => {
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
        CloudProviderType::GoogleDrive => return Err(SyncudioError::GoogleDrive("Google Drive not implemented yet".to_string())),
    }

    Ok(())
}