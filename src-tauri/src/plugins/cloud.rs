use std::path::PathBuf;
use log::{error, info};
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

use crate::libs::cloud::{CloudAuth, CloudFile, CloudProvider};
use crate::libs::cloud::dropbox::Dropbox;

// Dropbox commands
#[tauri::command]
pub async fn dropbox_start_auth(
    provider: tauri::State<'_, Dropbox>,
) -> Result<String, String> {
    info!("Handling cloud_dropbox_start_auth command");
    provider.start_authorization().await
}

#[tauri::command]
pub async fn dropbox_complete_auth(
    auth_code: String,
    provider: tauri::State<'_, Dropbox>,
) -> Result<CloudAuth, String> {
    info!(
        "Handling cloud_dropbox_complete_auth command with auth code: {}",
        auth_code
    );
    provider.complete_authorization(&auth_code).await
}

#[tauri::command]
pub async fn dropbox_is_authorized(provider: tauri::State<'_, Dropbox>) -> Result<bool, String> {
    Ok(provider.is_authorized().await)
}

#[tauri::command]
pub async fn dropbox_unauthorize(provider: tauri::State<'_, Dropbox>) -> Result<(), String> {
    provider.unauthorize().await;
    Ok(())
}

#[tauri::command]
pub async fn dropbox_list_files(
    provider: tauri::State<'_, Dropbox>,
    folder_id: String,
) -> Result<Vec<CloudFile>, String> {
    provider.list_files(&folder_id).await
}

#[tauri::command]
pub async fn dropbox_list_files_recursive(
    provider: tauri::State<'_, Dropbox>,
    folder_id: String,
) -> Result<Vec<CloudFile>, String> {
    provider.list_files_recursive(&folder_id).await
}

#[tauri::command]
pub async fn dropbox_create_folder(
    provider: tauri::State<'_, Dropbox>,
    name: String,
    parent_id: Option<String>,
) -> Result<CloudFile, String> {
    let folder = provider.create_folder(&name, parent_id.as_deref()).await?;
    Ok(CloudFile {
        id: folder.id,
        name: folder.name,
        parent_id: folder.parent_id,
        size: 0,
        is_folder: true,
        modified_at: 0,
        created_at: 0,
        mime_type: None,
        hash: None,
    })
}

#[tauri::command]
pub async fn dropbox_upload_file(
    provider: tauri::State<'_, Dropbox>,
    abs_local_path: String,
    name: String,
    parent_id: Option<String>,
) -> Result<CloudFile, String> {
    provider.upload_file(&PathBuf::from(abs_local_path), &name, parent_id.as_deref()).await
}

#[tauri::command]
pub async fn dropbox_download_file(
    provider: tauri::State<'_, Dropbox>,
    file_id: String,
    abs_local_path: String,
) -> Result<(), String> {
    provider.download_file(&file_id, &PathBuf::from(abs_local_path)).await
}

#[tauri::command]
pub async fn dropbox_delete_file(
    provider: tauri::State<'_, Dropbox>,
    file_id: String,
) -> Result<(), String> {
    provider.delete_file(&file_id).await
}

/**
 * Cloud plugin
 */
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("cloud") 
        .invoke_handler(tauri::generate_handler![
            dropbox_start_auth,
            dropbox_complete_auth,
            dropbox_is_authorized,
            dropbox_unauthorize,
            dropbox_list_files,
            dropbox_list_files_recursive,
            dropbox_create_folder,
            dropbox_upload_file,
            dropbox_download_file,
            dropbox_delete_file,
        ])
        .setup(move |app_handle, _api| {
            let app_handle = app_handle.clone();

            let dropbox = Dropbox::new();
            app_handle.manage(dropbox);

            Ok(())
        })
        .build()
}
