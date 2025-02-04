mod commands;
mod models;
mod providers;
mod database;

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
            get_cloud_music_folders,
            get_cloud_music_folders_by_provider,
            get_cloud_folder_by_local_path,
            save_cloud_folder,
            update_cloud_folder,
            delete_cloud_folder,
            // Cloud sync operations
            cleanup_missing_local_tracks,
            scan_cloud_music_folder,
            pull_cloud_metadata,
            push_cloud_metadata,
            get_cloud_folder_sync_details,
            get_queue_items,
            get_queue_stats,
            retry_failed_items,
            add_to_upload_queue,
            add_to_download_queue,
            get_track_sync_status,
            // Unified track commands
            get_unified_tracks,
            get_unified_tracks_by_ids,
            get_unified_tracks_by_folder,
            get_unified_tracks_by_provider,
            get_unified_track,
            // Sync worker commands
            reset_in_progress_items,
            get_next_upload_item,
            get_next_download_item,
            start_upload,
            start_download,
            fail_upload,
            fail_download,
            check_file_exists,
        ])
        .setup(move |app_handle, _api| {
            let dropbox = Dropbox::new();
            app_handle.manage(CloudState { dropbox });
            
            Ok(())
        })
        .build()
} 