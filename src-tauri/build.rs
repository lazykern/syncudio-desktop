fn main() {
    // Build the app
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .codegen(tauri_build::CodegenContext::new())
            .plugin(
                "app-menu",
                tauri_build::InlinedPlugin::new().commands(&["toggle"]),
            )
            .plugin(
                "config",
                tauri_build::InlinedPlugin::new().commands(&[
                    "get_storage_dir",
                    "get_config",
                    "set_config",
                ]),
            )
            .plugin(
                "cover",
                tauri_build::InlinedPlugin::new().commands(&["get_cover"]),
            )
            .plugin(
                "database",
                tauri_build::InlinedPlugin::new().commands(&[
                    "import_tracks_to_library",
                    "get_all_tracks",
                    "remove_tracks",
                    "get_tracks",
                    "update_track",
                    "get_all_playlists",
                    "get_playlist",
                    "create_playlist",
                    "rename_playlist",
                    "set_playlist_tracks",
                    "export_playlist",
                    "delete_playlist",
                    "reset",
                ]),
            )
            .plugin(
                "default-view",
                tauri_build::InlinedPlugin::new().commands(&["set"]),
            )
            .plugin(
                "sleepblocker",
                tauri_build::InlinedPlugin::new().commands(&["enable", "disable"]),
            )
            .plugin(
                "cloud",
                tauri_build::InlinedPlugin::new().commands(&[
                    // Dropbox-specific auth commands
                    "dropbox_start_auth",
                    "dropbox_complete_auth",
                    "dropbox_is_authorized",
                    "dropbox_unauthorize",
                    // Generic cloud operations
                    "cloud_list_files",
                    "cloud_list_root_files",
                    "cloud_create_folder",
                    "cloud_upload_file",
                    "cloud_download_file",
                    "cloud_delete_file",
                    // Cloud folder database operations
                    "get_cloud_folder",
                    "get_cloud_folders_by_provider",
                    "get_cloud_folder_by_local_path",
                    "save_cloud_folder",
                    "update_cloud_folder",
                    "delete_cloud_folder",
                    // Cloud track operations
                    "discover_cloud_folder_tracks",
                    "sync_cloud_tracks_metadata",
                ]),
            ),
    )
    .expect("Failed to run tauri-build");
}
