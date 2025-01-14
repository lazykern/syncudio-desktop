use std::collections::HashSet;
use std::path::PathBuf;
use log::{info, warn};
use tauri::{Runtime, State};
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::libs::constants::SUPPORTED_PLAYLISTS_EXTENSIONS;
use crate::libs::error::{AnyResult, SyncudioError};
use crate::libs::playlist::Playlist;
use crate::libs::track::get_track_id_for_path;
use crate::libs::utils::scan_dirs;

use super::core::DBState;

/// Get all playlists from the database
#[tauri::command]
pub async fn get_all_playlists(db_state: State<'_, DBState>) -> AnyResult<Vec<Playlist>> {
    db_state.get_lock().await.get_all_playlists().await
}

/// Get a specific playlist by ID
#[tauri::command]
pub async fn get_playlist(db_state: State<'_, DBState>, id: String) -> AnyResult<Playlist> {
    match db_state.get_lock().await.get_playlist(&id).await {
        Ok(Some(playlist)) => Ok(playlist),
        Ok(None) => Err(SyncudioError::PlaylistNotFound),
        Err(err) => Err(err),
    }
}

/// Create a new playlist
#[tauri::command]
pub async fn create_playlist(
    db_state: State<'_, DBState>,
    name: String,
    ids: Vec<String>,
    import_path: Option<PathBuf>,
) -> AnyResult<Playlist> {
    db_state
        .get_lock()
        .await
        .create_playlist(name, ids, import_path)
        .await
}

/// Rename an existing playlist
#[tauri::command]
pub async fn rename_playlist(
    db_state: State<'_, DBState>,
    id: String,
    name: String,
) -> AnyResult<Playlist> {
    db_state.get_lock().await.rename_playlist(&id, name).await
}

/// Update the tracks in a playlist
#[tauri::command]
pub async fn set_playlist_tracks(
    db_state: State<'_, DBState>,
    id: String,
    tracks: Vec<String>,
) -> AnyResult<Playlist> {
    db_state
        .get_lock()
        .await
        .set_playlist_tracks(&id, tracks)
        .await
}

/// Export a playlist to a file
#[tauri::command]
pub async fn export_playlist<R: Runtime>(
    window: tauri::Window<R>,
    db_state: State<'_, DBState>,
    id: String,
) -> AnyResult<()> {
    let mut db = db_state.get_lock().await;

    let Some(playlist) = db.get_playlist(&id).await? else {
        return Ok(());
    };

    let tracks = db.get_tracks(&playlist.tracks).await?;

    window
        .dialog()
        .file()
        .add_filter("playlist", &SUPPORTED_PLAYLISTS_EXTENSIONS)
        .save_file(move |maybe_playlist_path| {
            let playlist_path = match maybe_playlist_path {
                // We don't support FilePath::Url
                Some(FilePath::Path(path)) => path,
                _ => return,
            };

            let playlist_dir_path = playlist_path.parent().unwrap();

            let playlist = tracks
                .iter()
                .map(|track| {
                    let relative_path =
                        pathdiff::diff_paths(&track.path(), playlist_dir_path).unwrap();
                    m3u::path_entry(relative_path)
                })
                .collect::<Vec<m3u::Entry>>();

            let mut file = std::fs::File::create(playlist_path).unwrap();
            let mut writer = m3u::Writer::new(&mut file);
            for entry in &playlist {
                writer.write_entry(entry).unwrap();
            }
        });

    Ok(())
}

/// Delete a playlist
#[tauri::command]
pub async fn delete_playlist(db_state: State<'_, DBState>, id: String) -> AnyResult<()> {
    db_state.get_lock().await.delete_playlist(&id).await
}