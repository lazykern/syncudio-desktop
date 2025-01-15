use log::{error, info};
use std::path::PathBuf;
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime, State};

use crate::plugins::cloud::models::*;
use crate::plugins::cloud::providers::*;
use crate::libs::constants::{CLOUD_PROVIDER_DROPBOX, CLOUD_PROVIDER_GDRIVE};

// Dropbox-specific auth commands
#[tauri::command]
pub async fn dropbox_start_auth(provider: State<'_, Dropbox>) -> Result<String, String> {
    info!("Starting Dropbox authorization");
    provider.start_authorization().await
}

#[tauri::command]
pub async fn dropbox_complete_auth(
    auth_code: String,
    provider: State<'_, Dropbox>,
) -> Result<(), String> {
    info!(
        "Completing Dropbox authorization with auth code: {}",
        auth_code
    );
    let auth_data = provider.complete_authorization(&auth_code).await?;
    if let Some(auth_data) = auth_data {
        info!(
            "Dropbox authorization completed successfully: {:?}",
            auth_data
        );
    } else {
        error!("Dropbox authorization failed");
    }
    Ok(())
}

#[tauri::command]
pub async fn dropbox_is_authorized(provider: State<'_, Dropbox>) -> Result<bool, String> {
    Ok(provider.is_authorized().await)
}

#[tauri::command]
pub async fn dropbox_unauthorize(provider: State<'_, Dropbox>) -> Result<(), String> {
    provider.unauthorize().await;
    Ok(())
}

// Generic cloud file operation commands
#[tauri::command]
pub async fn cloud_list_files(
    provider_type: String,
    folder_id: String,
    recursive: bool,
    dropbox: State<'_, Dropbox>,
    // Add other providers here when implemented
) -> Result<Vec<CloudFile>, String> {
    match provider_type.as_str() {
        CLOUD_PROVIDER_DROPBOX => dropbox.list_files(&folder_id, recursive).await,
        CLOUD_PROVIDER_GDRIVE => Err("Google Drive not implemented yet".to_string()),
        _ => Err(format!("Unknown provider type: {}", provider_type)),
    }
}

#[tauri::command]
pub async fn cloud_list_root_files(
    provider_type: String,
    recursive: bool,
    dropbox: State<'_, Dropbox>,
    // Add other providers here when implemented
) -> Result<Vec<CloudFile>, String> {
    match provider_type.as_str() {
        CLOUD_PROVIDER_DROPBOX => dropbox.list_root_files(recursive).await,
        CLOUD_PROVIDER_GDRIVE => Err("Google Drive not implemented yet".to_string()),
        _ => Err(format!("Unknown provider type: {}", provider_type)),
    }
}

#[tauri::command]
pub async fn cloud_create_folder(
    provider_type: String,
    name: String,
    parent_id: Option<String>,
    dropbox: State<'_, Dropbox>,
    // Add other providers here when implemented
) -> Result<CloudFile, String> {
    match provider_type.as_str() {
        CLOUD_PROVIDER_DROPBOX => {
            let folder = dropbox.create_folder(&name, parent_id.as_deref()).await?;
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
        CLOUD_PROVIDER_GDRIVE => Err("Google Drive not implemented yet".to_string()),
        _ => Err(format!("Unknown provider type: {}", provider_type)),
    }
}

#[tauri::command]
pub async fn cloud_upload_file(
    provider_type: String,
    abs_local_path: String,
    name: String,
    parent_id: Option<String>,
    dropbox: State<'_, Dropbox>,
    // Add other providers here when implemented
) -> Result<CloudFile, String> {
    match provider_type.as_str() {
        CLOUD_PROVIDER_DROPBOX => {
            dropbox
                .upload_file(&PathBuf::from(abs_local_path), &name, parent_id.as_deref())
                .await
        }
        CLOUD_PROVIDER_GDRIVE => Err("Google Drive not implemented yet".to_string()),
        _ => Err(format!("Unknown provider type: {}", provider_type)),
    }
}

#[tauri::command]
pub async fn cloud_download_file(
    provider_type: String,
    file_id: String,
    abs_local_path: String,
    dropbox: State<'_, Dropbox>,
    // Add other providers here when implemented
) -> Result<(), String> {
    match provider_type.as_str() {
        CLOUD_PROVIDER_DROPBOX => {
            dropbox
                .download_file(&file_id, &PathBuf::from(abs_local_path))
                .await
        }
        CLOUD_PROVIDER_GDRIVE => Err("Google Drive not implemented yet".to_string()),
        _ => Err(format!("Unknown provider type: {}", provider_type)),
    }
}

#[tauri::command]
pub async fn cloud_delete_file(
    provider_type: String,
    file_id: String,
    dropbox: State<'_, Dropbox>,
    // Add other providers here when implemented
) -> Result<(), String> {
    match provider_type.as_str() {
        CLOUD_PROVIDER_DROPBOX => dropbox.delete_file(&file_id).await,
        CLOUD_PROVIDER_GDRIVE => Err("Google Drive not implemented yet".to_string()),
        _ => Err(format!("Unknown provider type: {}", provider_type)),
    }
}

/**
 * Cloud plugin
 */
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("cloud")
        .invoke_handler(tauri::generate_handler![
            // Dropbox-specific auth commands
            dropbox_start_auth,
            dropbox_complete_auth,
            dropbox_is_authorized,
            dropbox_unauthorize,
            // Generic cloud operations
            cloud_list_files,
            cloud_list_root_files,
            cloud_create_folder,
            cloud_upload_file,
            cloud_download_file,
            cloud_delete_file,
        ])
        .setup(move |app_handle, _api| {

            let dropbox = Dropbox::new();
            app_handle.manage(dropbox);

            Ok(())
        })
        .build()
}