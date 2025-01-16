use log::{info, error};
use std::path::PathBuf;
use tauri::State;

use crate::libs::error::AnyResult;
use crate::libs::error::SyncudioError;
use crate::plugins::cloud::providers::Dropbox;
use crate::plugins::cloud::providers::CloudProviderType;
use crate::plugins::cloud::CloudState;
use crate::plugins::cloud::{CloudFile, CloudProvider};

// Dropbox-specific auth commands
#[tauri::command]
pub async fn dropbox_start_auth(provider: State<'_, Dropbox>) -> AnyResult<String> {
    info!("Starting Dropbox authorization");
    provider.start_authorization().await
}

#[tauri::command]
pub async fn dropbox_complete_auth(
    auth_code: String,
    provider: State<'_, Dropbox>,
) -> AnyResult<()> {
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
pub async fn dropbox_is_authorized(provider: State<'_, Dropbox>) -> AnyResult<bool> {
    Ok(provider.is_authorized().await)
}

#[tauri::command]
pub async fn dropbox_unauthorize(provider: State<'_, Dropbox>) -> AnyResult<()> {
    provider.unauthorize().await;
    Ok(())
}

// Generic cloud file operation commands
#[tauri::command]
pub async fn cloud_list_files(
    provider_type: String,
    folder_id: String,
    recursive: bool,
    cloud_state: State<'_, CloudState>,
    // Add other providers here when implemented
) -> AnyResult<Vec<CloudFile>> {
    let provider = CloudProviderType::from_str(&provider_type)?;

    match provider {
        CloudProviderType::Dropbox => cloud_state.dropbox.list_files(&folder_id, recursive).await,
        CloudProviderType::GoogleDrive => Err(SyncudioError::GoogleDrive("Google Drive not implemented yet".to_string())),
    }
}

#[tauri::command]
pub async fn cloud_list_root_files(
    provider_type: String,
    recursive: bool,
    cloud_state: State<'_, CloudState>,
    // Add other providers here when implemented
) -> AnyResult<Vec<CloudFile>> {
    let provider = CloudProviderType::from_str(&provider_type)?;

    match provider {
        CloudProviderType::Dropbox => cloud_state.dropbox.list_root_files(recursive).await,
        CloudProviderType::GoogleDrive => Err(SyncudioError::GoogleDrive("Google Drive not implemented yet".to_string())),
    }
}

#[tauri::command]
pub async fn cloud_create_folder(
    provider_type: String,
    name: String,
    parent_id: Option<String>,
    cloud_state: State<'_, CloudState>,
    // Add other providers here when implemented
) -> AnyResult<CloudFile> {
    let provider = CloudProviderType::from_str(&provider_type)?;

    match provider {
        CloudProviderType::Dropbox => {
            let folder = cloud_state.dropbox.create_folder(&name, parent_id.as_deref()).await?;
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
        CloudProviderType::GoogleDrive => Err(SyncudioError::GoogleDrive("Google Drive not implemented yet".to_string())),
    }
}

#[tauri::command]
pub async fn cloud_upload_file(
    provider_type: String,
    abs_local_path: String,
    name: String,
    parent_id: Option<String>,
    cloud_state: State<'_, CloudState>,
    // Add other providers here when implemented
) -> AnyResult<CloudFile> {
    let provider = CloudProviderType::from_str(&provider_type)?;

    match provider {
        CloudProviderType::Dropbox => {
            cloud_state.dropbox
                .upload_file(&PathBuf::from(abs_local_path), &name, parent_id.as_deref())
                .await
        }
        CloudProviderType::GoogleDrive => Err(SyncudioError::GoogleDrive("Google Drive not implemented yet".to_string())),
    }
}

#[tauri::command]
pub async fn cloud_download_file(
    provider_type: String,
    file_id: String,
    abs_local_path: String,
    cloud_state: State<'_, CloudState>,
    // Add other providers here when implemented
) -> AnyResult<()> {
    let provider = CloudProviderType::from_str(&provider_type)?;

    match provider {
        CloudProviderType::Dropbox => {
            cloud_state.dropbox
                .download_file(&file_id, &PathBuf::from(abs_local_path))
                .await
        }
        CloudProviderType::GoogleDrive => Err(SyncudioError::GoogleDrive("Google Drive not implemented yet".to_string())),
    }
}

#[tauri::command]
pub async fn cloud_delete_file(
    provider_type: String,
    file_id: String,
    cloud_state: State<'_, CloudState>,
    // Add other providers here when implemented
) -> AnyResult<()> {
    let provider = CloudProviderType::from_str(&provider_type)?;

    match provider {
        CloudProviderType::Dropbox => cloud_state.dropbox.delete_file(&file_id).await,
        CloudProviderType::GoogleDrive => Err(SyncudioError::GoogleDrive("Google Drive not implemented yet".to_string())),
    }
}