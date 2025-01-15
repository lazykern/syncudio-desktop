use ormlite::sqlite::SqliteConnection;
use crate::libs::error::AnyResult;


pub async fn create_tables(connection: &mut SqliteConnection) -> AnyResult<()> {
    // Cloud provider table - using provider_type as primary key since we only allow one connection per provider
    ormlite::query(
        "CREATE TABLE IF NOT EXISTS cloud_providers (
            provider_type TEXT PRIMARY KEY NOT NULL, -- 'dropbox' or 'gdrive'
            auth_data TEXT -- Raw auth data for provider-specific storage
        );",
    )
    .execute(&mut *connection)
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
    .execute(&mut *connection)
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
    .execute(&mut *connection)
    .await?;

    Ok(())
} 