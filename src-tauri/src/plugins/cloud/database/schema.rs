use crate::libs::error::AnyResult;
use ormlite::sqlite::SqliteConnection;

pub async fn create_tables(connection: &mut SqliteConnection) -> AnyResult<()> {
    // Cloud folder mappings
    ormlite::query(
        "CREATE TABLE IF NOT EXISTS cloud_folders (
            id TEXT PRIMARY KEY NOT NULL,
            provider_type TEXT NOT NULL,
            cloud_folder_id TEXT NOT NULL,
            cloud_folder_path TEXT NOT NULL,
            local_folder_path TEXT NOT NULL UNIQUE
        );",
    )
    .execute(&mut *connection)
    .await?;

    // Cloud tracks table
    ormlite::query(
        "CREATE TABLE IF NOT EXISTS cloud_tracks (
            id TEXT PRIMARY KEY NOT NULL,
            blake3_hash TEXT UNIQUE,
            old_blake3_hashes JSON NOT NULL, -- JSON array of old hashes
            cloud_file_id TEXT UNIQUE,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            file_name TEXT NOT NULL,
            tags JSON -- JSON object of CloudTrackTag
        );",
    )
    .execute(&mut *connection)
    .await?;

    // Create indexes for performance
    ormlite::query(
        "CREATE INDEX IF NOT EXISTS idx_cloud_tracks_blake3_hash ON cloud_tracks(blake3_hash);
         CREATE INDEX IF NOT EXISTS idx_cloud_tracks_cloud_file_id ON cloud_tracks(cloud_file_id);",
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}
