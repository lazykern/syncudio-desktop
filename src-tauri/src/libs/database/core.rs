use ormlite::sqlite::SqliteConnection;

use crate::libs::error::AnyResult;

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
                blake3_hash TEXT,
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

        // Index for the path column in Track
        ormlite::query("CREATE INDEX IF NOT EXISTS index_track_path ON tracks (path);")
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

        // Create cloud-related tables
        crate::plugins::cloud::create_tables(&mut self.connection).await?;

        Ok(())
    }
} 