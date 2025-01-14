use tauri::Runtime;
use tauri::State;

use crate::libs::error::AnyResult;
use crate::libs::local_folder::LocalFolder;
use crate::plugins::db::DBState;

/// Get all local folders
#[tauri::command]
pub async fn get_local_folders(db_state: State<'_, DBState>) -> AnyResult<Vec<LocalFolder>> {
    db_state.get_lock().await.get_all_local_folders().await
}

/// Add new local folders to the library
#[tauri::command]
pub async fn add_local_folder(db_state: State<'_, DBState>, path: &str) -> AnyResult<LocalFolder> {
    db_state.get_lock().await.create_local_folder(path.to_string()).await
}

/// Remove a local folder from the library
#[tauri::command]
pub async fn remove_local_folder(db_state: State<'_, DBState>, path: &str) -> AnyResult<()> {
    db_state.get_lock().await.delete_local_folder(path).await
}

/// Check if a path is a registered local folder
#[tauri::command]
pub async fn is_local_folder(db_state: State<'_, DBState>, path: &str) -> AnyResult<bool> {
    db_state.get_lock().await.get_local_folder(path).await.map(|f| f.is_some())
} 