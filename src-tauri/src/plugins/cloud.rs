use dropbox_sdk::oauth2::PkceCode;
use log::{error, info};
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

use crate::libs::cloud::dropbox::Dropbox;

#[tauri::command]
pub async fn dropbox_start_auth(
    provider: tauri::State<'_, Dropbox>,
) -> Result<String, String> {
    info!("Handling cloud_dropbox_start_auth command");
    let pkce_code = PkceCode::new();
    provider.start_authorization(pkce_code)
        .await
        .map_err(|e| {
            error!("Failed to get authorization URL: {}", e);
            e.to_string()
        })
}

#[tauri::command]
pub async fn dropbox_complete_auth(
    auth_code: String,
    provider: tauri::State<'_, Dropbox>,
) -> Result<(String, Option<String>), String> {
    info!(
        "Handling cloud_dropbox_complete_auth command with auth code: {}",
        auth_code
    );
    let result = provider.complete_authorization(&auth_code).await;
    match &result {
        Ok((token, auth_data)) => {
            info!("Authorization completed successfully");
            info!("Access Token: {}", token);
            info!("Auth Data: {:?}", auth_data);
        }
        Err(e) => {
            error!("Authorization failed: {}", e);
        }
    }
    result
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
        ])
        .setup(move |app_handle, _api| {
            let app_handle = app_handle.clone();

            let dropbox = Dropbox::new();

            app_handle.manage(dropbox);

            Ok(())
        })
        .build()
}
