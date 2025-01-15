use log::info;
use tauri::State;

use crate::libs::error::AnyResult;
use crate::plugins::cloud::models::cloud_folder::CloudFolder;
use crate::plugins::db::DBState;

// Cloud Folder Operations
#[tauri::command]
pub async fn get_cloud_folder(id: String, db_state: State<'_, DBState>) -> AnyResult<Option<CloudFolder>> {
    info!("Getting cloud folder with id: {}", id);
    let mut db = db_state.get_lock().await;
    db.get_cloud_folder(&id).await
}

#[tauri::command]
pub async fn get_cloud_folders_by_provider(provider_type: String, db_state: State<'_, DBState>) -> AnyResult<Vec<CloudFolder>> {
    info!("Getting cloud folders for provider: {}", provider_type);
    let mut db = db_state.get_lock().await;
    db.get_cloud_folders_by_provider(&provider_type).await
}

#[tauri::command]
pub async fn get_cloud_folder_by_local_path(local_path: String, db_state: State<'_, DBState>) -> AnyResult<Option<CloudFolder>> {
    info!("Getting cloud folder for local path: {}", local_path);
    let mut db = db_state.get_lock().await;
    db.get_cloud_folder_by_local_path(&local_path).await
}

#[tauri::command]
pub async fn save_cloud_folder(folder: CloudFolder, db_state: State<'_, DBState>) -> AnyResult<CloudFolder> {
    info!("Saving cloud folder: {:?}", folder);
    let mut db = db_state.get_lock().await;
    db.save_cloud_folder(folder).await
}

#[tauri::command]
pub async fn update_cloud_folder(folder: CloudFolder, db_state: State<'_, DBState>) -> AnyResult<CloudFolder> {
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
