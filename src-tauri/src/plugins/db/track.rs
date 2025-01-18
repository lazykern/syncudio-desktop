use tauri::State;

use crate::libs::error::AnyResult;
use crate::libs::track::Track;

use super::core::DBState;

/// Get all tracks from the database
#[tauri::command]
pub async fn get_all_tracks(db_state: State<'_, DBState>) -> AnyResult<Vec<Track>> {
    db_state.get_lock().await.get_all_tracks().await
}

/// Get specific tracks by their IDs
#[tauri::command]
pub async fn get_tracks(db_state: State<'_, DBState>, ids: Vec<String>) -> AnyResult<Vec<Track>> {
    db_state.get_lock().await.get_tracks(&ids).await
}

/// Update a track in the database
#[tauri::command]
pub async fn update_track(db_state: State<'_, DBState>, track: Track) -> AnyResult<Track> {
    db_state.get_lock().await.update_track(track).await
}

/// Remove tracks from the database
#[tauri::command]
pub async fn remove_tracks(db_state: State<'_, DBState>, ids: Vec<String>) -> AnyResult<()> {
    db_state.get_lock().await.remove_tracks(&ids).await
} 