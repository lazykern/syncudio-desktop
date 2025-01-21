use log::info;
use tauri::State;

use crate::libs::error::AnyResult;
use crate::plugins::cloud::models::cloud_folder::CloudMusicFolder;
use crate::plugins::db::DBState;

// Cloud Folder Operations
#[tauri::command]
pub async fn get_cloud_folders(db_state: State<'_, DBState>) -> AnyResult<Vec<CloudMusicFolder>> {
    info!("Getting all cloud folders");
    let mut db = db_state.get_lock().await;
    db.get_cloud_folders().await
}

#[tauri::command]
pub async fn get_cloud_folders_by_provider(provider_type: String, db_state: State<'_, DBState>) -> AnyResult<Vec<CloudMusicFolder>> {
    info!("Getting cloud folders for provider: {}", provider_type);
    let mut db = db_state.get_lock().await;
    db.get_cloud_folders_by_provider(&provider_type).await
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
