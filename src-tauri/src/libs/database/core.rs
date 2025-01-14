use ormlite::model::ModelBuilder;
use ormlite::sqlite::SqliteConnection;
use ormlite::Model;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::libs::error::AnyResult;
use crate::libs::playlist::Playlist;
use crate::libs::track::Track;
use crate::libs::utils::TimeLogger;

/// Core database struct that holds the SQLite connection
pub struct DB {
    pub connection: SqliteConnection,
}

impl DB {
    /// Create tables within a SQLite connection
    pub async fn create_tables(&mut self) -> AnyResult<()> {
        // TODO: move that to SQL files, or derive that from the struct itself, probably need to create a PR for ormlite-cli
        ormlite::query(
            "CREATE TABLE IF NOT EXISTS tracks (
                id TEXT PRIMARY KEY NOT NULL,
                path TEXT NOT NULL UNIQUE, -- Path as a string and unique
                title TEXT NOT NULL,
                album TEXT NOT NULL,
                artists JSON NOT NULL, -- Array of strings
                genres JSON NOT NULL, -- Array of strings
                year INTEGER,
                duration INTEGER NOT NULL,
                track_no INTEGER,
                track_of INTEGER,
                disk_no INTEGER,
                disk_of INTEGER
            );",
        )
        .execute(&mut self.connection)
        .await?;

        // Cloud provider table - using provider_type as primary key since we only allow one connection per provider
        ormlite::query(
            "CREATE TABLE IF NOT EXISTS cloud_providers (
                provider_type TEXT PRIMARY KEY NOT NULL, -- 'dropbox' or 'gdrive'
                auth_data TEXT -- Raw auth data for provider-specific storage
            );",
        )
        .execute(&mut self.connection)
        .await?;

        // Cloud folder mappings
        ormlite::query(
            "CREATE TABLE IF NOT EXISTS cloud_folders (
                id TEXT PRIMARY KEY NOT NULL,
                provider_type TEXT NOT NULL, -- Reference to cloud_providers
                cloud_folder_id TEXT NOT NULL,
                cloud_folder_name TEXT NOT NULL,
                local_folder_path TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY(provider_type) REFERENCES cloud_providers(provider_type)
            );",
        )
        .execute(&mut self.connection)
        .await?;

        // Cloud sync status for tracks and playlists
        ormlite::query(
            "CREATE TABLE IF NOT EXISTS cloud_syncs (
                id TEXT PRIMARY KEY NOT NULL,
                provider_type TEXT NOT NULL, -- Reference to cloud_providers
                folder_id TEXT NOT NULL, -- Reference to cloud_folders
                item_id TEXT NOT NULL, -- track_id or playlist_id
                item_type TEXT NOT NULL, -- 'track' or 'playlist'
                cloud_file_id TEXT NOT NULL,
                cloud_file_name TEXT NOT NULL,
                local_path TEXT NOT NULL,
                last_synced INTEGER,
                sync_status TEXT NOT NULL, -- 'synced', 'pending_upload', 'pending_download', 'conflict'
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY(provider_type) REFERENCES cloud_providers(provider_type),
                FOREIGN KEY(folder_id) REFERENCES cloud_folders(id)
            );",
        )
        .execute(&mut self.connection)
        .await?;

        // Index for the path column in Track
        ormlite::query("CREATE INDEX IF NOT EXISTS index_track_path ON tracks (path);")
            .execute(&mut self.connection)
            .await?;

        // Indices for cloud tables
        ormlite::query("CREATE INDEX IF NOT EXISTS index_cloud_folder_provider ON cloud_folders (provider_type);")
            .execute(&mut self.connection)
            .await?;

        ormlite::query("CREATE INDEX IF NOT EXISTS index_cloud_folder_path ON cloud_folders (local_folder_path);")
            .execute(&mut self.connection)
            .await?;

        ormlite::query("CREATE INDEX IF NOT EXISTS index_cloud_syncs_item ON cloud_syncs (item_id, item_type);")
            .execute(&mut self.connection)
            .await?;

        ormlite::query("CREATE INDEX IF NOT EXISTS index_cloud_syncs_folder ON cloud_syncs (folder_id);")
            .execute(&mut self.connection)
            .await?;

        ormlite::query("CREATE INDEX IF NOT EXISTS index_cloud_syncs_status ON cloud_syncs (sync_status);")
            .execute(&mut self.connection)
            .await?;

        ormlite::query(
            "CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                tracks JSON NOT NULL DEFAULT '[]', -- Array of track IDs
                import_path TEXT UNIQUE -- Path of the playlist file, unique if it exists
            );",
        )
        .execute(&mut self.connection)
        .await?;

        Ok(())
    }
} 