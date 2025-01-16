mod commands;
mod models;
mod providers;
mod database;
use log::error;
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

use crate::plugins::cloud::providers::*;

pub use commands::*;
pub use models::*;
pub use database::*;

pub struct CloudState {
    pub dropbox: Dropbox,
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
            // Cloud folder database operations
            get_cloud_folder,
            get_cloud_folders_by_provider,
            get_cloud_folder_by_local_path,
            save_cloud_folder,
            update_cloud_folder,
            delete_cloud_folder,
        ])
        .setup(move |app_handle, _api| {
            let dropbox = Dropbox::new();
            app_handle.manage(CloudState { dropbox });
            Ok(())
        })
        .build()
} 