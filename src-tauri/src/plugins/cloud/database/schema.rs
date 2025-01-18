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
            cloud_file_id TEXT UNIQUE,
            old_blake3_hashes JSON NOT NULL, -- JSON array of old hashes
            updated_at DATETIME NOT NULL,
            file_name TEXT NOT NULL,
            tags JSON -- JSON object of CloudTrackTag
        );",
    )
    .execute(&mut *connection)
    .await?;

    // Cloud track paths table
    ormlite::query(
        "CREATE TABLE IF NOT EXISTS cloud_track_maps (
            id TEXT PRIMARY KEY NOT NULL,
            cloud_track_id TEXT NOT NULL,
            cloud_folder_id TEXT NOT NULL,
            relative_path TEXT NOT NULL,
            FOREIGN KEY (cloud_track_id) REFERENCES cloud_tracks(id),
            FOREIGN KEY (cloud_folder_id) REFERENCES cloud_folders(id)
        );
        CREATE INDEX IF NOT EXISTS idx_cloud_track_maps_cloud_track_id ON cloud_track_maps(cloud_track_id);
        CREATE INDEX IF NOT EXISTS idx_cloud_track_maps_cloud_folder_id ON cloud_track_maps(cloud_folder_id);
        CREATE INDEX IF NOT EXISTS idx_cloud_track_maps_folder_track ON cloud_track_maps(cloud_folder_id, cloud_track_id);",
    )
    .execute(&mut *connection)
    .await?;

    // Download queue table
    ormlite::query(
        "CREATE TABLE IF NOT EXISTS download_queue (
            id TEXT PRIMARY KEY NOT NULL,
            cloud_track_map_id TEXT NOT NULL,
            provider_type TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL,
            error_message TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            attempts INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (cloud_track_map_id) REFERENCES cloud_track_maps(id)
        );"
    )
    .execute(&mut *connection)
    .await?;

    // Create indexes for download queue
    ormlite::query(
        "CREATE INDEX IF NOT EXISTS idx_download_queue_cloud_track_map_id ON download_queue(cloud_track_map_id);
         CREATE INDEX IF NOT EXISTS idx_download_queue_status ON download_queue(status);
         CREATE INDEX IF NOT EXISTS idx_download_queue_map_status ON download_queue(cloud_track_map_id, status);"
    )
    .execute(&mut *connection)
    .await?;

    // Add partial unique index for active download operations
    ormlite::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_download_queue_active_ops ON download_queue(cloud_track_map_id) 
         WHERE status IN ('pending', 'in_progress');"
    )
    .execute(&mut *connection)
    .await?;

    // Upload queue table
    ormlite::query(
        "CREATE TABLE IF NOT EXISTS upload_queue (
            id TEXT PRIMARY KEY NOT NULL,
            cloud_track_map_id TEXT NOT NULL,
            provider_type TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL,
            error_message TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            attempts INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (cloud_track_map_id) REFERENCES cloud_track_maps(id)
        );"
    )
    .execute(&mut *connection)
    .await?;

    // Create indexes for upload queue
    ormlite::query(
        "CREATE INDEX IF NOT EXISTS idx_upload_queue_cloud_track_map_id ON upload_queue(cloud_track_map_id);
         CREATE INDEX IF NOT EXISTS idx_upload_queue_status ON upload_queue(status);
         CREATE INDEX IF NOT EXISTS idx_upload_queue_map_status ON upload_queue(cloud_track_map_id, status);"
    )
    .execute(&mut *connection)
    .await?;

    // Add partial unique index for active upload operations
    ormlite::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_upload_queue_active_ops ON upload_queue(cloud_track_map_id) 
         WHERE status IN ('pending', 'in_progress');"
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}
