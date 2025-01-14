use ormlite::{sqlite::SqliteConnection, Connection};

use crate::libs::error::AnyResult;

/// Core database struct that holds the SQLite connection
pub struct DB {
    pub connection: SqliteConnection,
}

impl DB {
    /// Create a new database connection
    pub async fn new() -> AnyResult<Self> {
        let connection = SqliteConnection::connect("syncudio.db").await?;
        let mut db = Self { connection };
        tokio::runtime::Runtime::new()?.block_on(db.create_tables())?;
        Ok(db)
    }

    /// Create tables within a SQLite connection
    pub async fn create_tables(&mut self) -> AnyResult<()> {
        // Local folders table
        ormlite::query(
            "CREATE TABLE IF NOT EXISTS local_folders (
                path TEXT PRIMARY KEY NOT NULL UNIQUE
            );",
        )
        .execute(&mut self.connection)
        .await?;

        // Tracks table
        ormlite::query(
            "CREATE TABLE IF NOT EXISTS tracks (
                id TEXT PRIMARY KEY NOT NULL,
                local_folder_path TEXT NOT NULL,
                relative_path TEXT NOT NULL,
                title TEXT NOT NULL,
                album TEXT NOT NULL,
                artists JSON NOT NULL,
                genres JSON NOT NULL,
                year INTEGER,
                duration INTEGER NOT NULL,
                track_no INTEGER,
                track_of INTEGER,
                disk_no INTEGER,
                disk_of INTEGER,
                index_hash TEXT,
                UNIQUE(local_folder_path, relative_path)
            );",
        )
        .execute(&mut self.connection)
        .await?;

        // Cloud provider table
        ormlite::query(
            "CREATE TABLE IF NOT EXISTS cloud_providers (
                provider_type TEXT PRIMARY KEY NOT NULL,
                auth_data TEXT
            );",
        )
        .execute(&mut self.connection)
        .await?;

        // Cloud folder mappings
        ormlite::query(
            "CREATE TABLE IF NOT EXISTS cloud_folders (
                id TEXT PRIMARY KEY NOT NULL,
                provider_type TEXT NOT NULL,
                cloud_folder_id TEXT NOT NULL,
                cloud_folder_name TEXT NOT NULL,
                local_folder_path TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY(provider_type) REFERENCES cloud_providers(provider_type),
                FOREIGN KEY(local_folder_path) REFERENCES local_folders(path)
            );",
        )
        .execute(&mut self.connection)
        .await?;

        // Cloud sync status for tracks and playlists
        ormlite::query(
            "CREATE TABLE IF NOT EXISTS cloud_syncs (
                id TEXT PRIMARY KEY NOT NULL,
                provider_type TEXT NOT NULL,
                folder_id TEXT NOT NULL,
                item_id TEXT NOT NULL,
                item_type TEXT NOT NULL,
                cloud_file_id TEXT NOT NULL,
                cloud_file_name TEXT NOT NULL,
                local_path TEXT NOT NULL,
                last_synced INTEGER,
                sync_status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY(provider_type) REFERENCES cloud_providers(provider_type),
                FOREIGN KEY(folder_id) REFERENCES cloud_folders(id)
            );",
        )
        .execute(&mut self.connection)
        .await?;

        // Playlists table
        ormlite::query(
            "CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                tracks JSON NOT NULL DEFAULT '[]',
                import_path TEXT UNIQUE
            );",
        )
        .execute(&mut self.connection)
        .await?;

        // Cloud playlists table - uses index_hash for track references
        ormlite::query(
            "CREATE TABLE IF NOT EXISTS cloud_playlists (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                tracks JSON NOT NULL DEFAULT '[]',  -- Array of {index_hash: string, added_at: number}
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                creator_id TEXT,                    -- For future user identification
                is_public BOOLEAN NOT NULL DEFAULT false
            );",
        )
        .execute(&mut self.connection)
        .await?;

        // Create indices
        ormlite::query("CREATE INDEX IF NOT EXISTS index_track_folder ON tracks (local_folder_path);")
            .execute(&mut self.connection)
            .await?;

        ormlite::query("CREATE INDEX IF NOT EXISTS index_track_path ON tracks (local_folder_path, relative_path);")
            .execute(&mut self.connection)
            .await?;

        ormlite::query("CREATE INDEX IF NOT EXISTS index_track_index_hash ON tracks (index_hash);")
            .execute(&mut self.connection)
            .await?;

        ormlite::query("CREATE INDEX IF NOT EXISTS index_local_folder_path ON local_folders (path);")
            .execute(&mut self.connection)
            .await?;

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

        Ok(())
    }
} 