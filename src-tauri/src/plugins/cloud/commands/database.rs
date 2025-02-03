use log::info;
use tauri::State;

use crate::libs::error::AnyResult;
use crate::plugins::cloud::models::*;
use crate::plugins::db::DBState;

// Cloud Folder Operations
#[tauri::command]
pub async fn get_cloud_music_folders(db_state: State<'_, DBState>) -> AnyResult<Vec<CloudMusicFolder>> {
    info!("Getting all cloud folders");
    let mut db = db_state.get_lock().await;
    db.get_cloud_music_folders().await
}

#[tauri::command]
pub async fn get_cloud_music_folders_by_provider(provider_type: String, db_state: State<'_, DBState>) -> AnyResult<Vec<CloudMusicFolder>> {
    info!("Getting cloud folders for provider: {}", provider_type);
    let mut db = db_state.get_lock().await;
    db.get_cloud_music_folders_by_provider(&provider_type).await
}

#[tauri::command]
pub async fn get_cloud_folder_by_local_path(local_path: String, db_state: State<'_, DBState>) -> AnyResult<Option<CloudMusicFolder>> {
    info!("Getting cloud folder for local path: {}", local_path);
    let mut db = db_state.get_lock().await;
    db.get_cloud_folder_by_local_path(&local_path).await
}

#[tauri::command]
pub async fn save_cloud_folder(folder: CloudMusicFolder, db_state: State<'_, DBState>) -> AnyResult<CloudMusicFolder> {
    info!("Saving cloud folder: {:?}", folder);
    let mut db = db_state.get_lock().await;
    db.save_cloud_folder(folder).await
}

#[tauri::command]
pub async fn update_cloud_folder(folder: CloudMusicFolder, db_state: State<'_, DBState>) -> AnyResult<CloudMusicFolder> {
    info!("Updating cloud folder: {:?}", folder);
    let mut db = db_state.get_lock().await;
    db.update_cloud_folder(folder).await
}

#[tauri::command]
pub async fn delete_cloud_folder(id: String, db_state: State<'_, DBState>) -> AnyResult<()> {
    info!("Deleting cloud folder with id: {}", id);
    let mut db = db_state.get_lock().await;
    db.delete_cloud_folder(&id).await
}

#[tauri::command]
pub async fn get_unified_tracks(db_state: State<'_, DBState>) -> AnyResult<Vec<UnifiedTrack>> {
    info!("Getting all unified tracks");
    let mut db = db_state.get_lock().await;
    db.get_unified_tracks().await
}

#[tauri::command]
pub async fn get_unified_tracks_by_ids(
    ids: Vec<String>,
    db_state: State<'_, DBState>
) -> AnyResult<Vec<UnifiedTrack>> {
    info!("Getting unified tracks by ids: {:?}", ids);
    let mut db = db_state.get_lock().await;
    db.get_unified_tracks_by_ids(&ids).await
}

#[tauri::command]
pub async fn get_unified_track(
    id: String,
    db_state: State<'_, DBState>
) -> AnyResult<Option<UnifiedTrack>> {
    info!("Getting unified track by id: {}", id);
    let mut db = db_state.get_lock().await;
    db.get_unified_track(&id).await
}

#[tauri::command]
pub async fn get_unified_tracks_by_folder(
    folder_id: String,
    db_state: State<'_, DBState>
) -> AnyResult<Vec<UnifiedTrack>> {
    info!("Getting unified tracks by folder id: {}", folder_id);
    let mut db = db_state.get_lock().await;
    db.get_unified_tracks_by_folder(&folder_id).await
}

#[tauri::command]
pub async fn get_unified_tracks_by_provider(
    provider_type: String,
    db_state: State<'_, DBState>
) -> AnyResult<Vec<UnifiedTrack>> {
    info!("Getting unified tracks by provider type: {}", provider_type);
    let mut db = db_state.get_lock().await;
    db.get_unified_tracks_by_provider(&provider_type).await
}
