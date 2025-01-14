use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use log::{error, info, warn};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, Runtime, State};
use tauri_plugin_dialog::DialogExt;
use ts_rs::TS;

use crate::libs::constants::SUPPORTED_TRACKS_EXTENSIONS;
use crate::libs::error::{AnyResult, SyncudioError};
use crate::libs::events::IPCEvent;
use crate::libs::track::{get_track_from_file, get_track_id_for_path, Track};
use crate::libs::utils::{scan_dirs, TimeLogger};

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