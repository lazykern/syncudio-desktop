use crate::libs::error::AnyResult;
use ormlite::sqlite::SqliteConnection;

pub async fn create_tables(connection: &mut SqliteConnection) -> AnyResult<()> {
    // Cloud folder mappings
    ormlite::query(
        "CREATE TABLE IF NOT EXISTS cloud_music_folders (
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
            file_name TEXT NOT NULL,
            updated_at DATETIME NOT NULL,
            tags JSON -- JSON object of CloudTrackTag
        );",
    )
    .execute(&mut *connection)
    .await?;

    // Cloud track maps table - maps tracks to their locations
    ormlite::query(
        "CREATE TABLE IF NOT EXISTS cloud_maps (
            id TEXT PRIMARY KEY NOT NULL,
            cloud_track_id TEXT NOT NULL,
            cloud_music_folder_id TEXT NOT NULL,
            relative_path TEXT NOT NULL,
            cloud_file_id TEXT UNIQUE, -- Moved from cloud_tracks to here since it's location-specific
            FOREIGN KEY (cloud_track_id) REFERENCES cloud_tracks(id),
            FOREIGN KEY (cloud_music_folder_id) REFERENCES cloud_music_folders(id)
        );
        CREATE INDEX IF NOT EXISTS idx_cloud_maps_cloud_track_id ON cloud_maps(cloud_track_id);
        CREATE INDEX IF NOT EXISTS idx_cloud_maps_cloud_music_folder_id ON cloud_maps(cloud_music_folder_id);
        CREATE INDEX IF NOT EXISTS idx_cloud_maps_folder_track ON cloud_maps(cloud_music_folder_id, cloud_track_id);",
    )
    .execute(&mut *connection)
    .await?;

    // Download queue table
    ormlite::query(
        "CREATE TABLE IF NOT EXISTS download_queue (
            id TEXT PRIMARY KEY NOT NULL,
            cloud_map_id TEXT NOT NULL,
            provider_type TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL,
            error_message TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            attempts INTEGER NOT NULL DEFAULT 0
        );"
    )
    .execute(&mut *connection)
    .await?;

    // Create indexes for download queue
    ormlite::query(
        "CREATE INDEX IF NOT EXISTS idx_download_queue_cloud_map_id ON download_queue(cloud_map_id);
         CREATE INDEX IF NOT EXISTS idx_download_queue_status ON download_queue(status);
         CREATE INDEX IF NOT EXISTS idx_download_queue_map_status ON download_queue(cloud_map_id, status);"
    )
    .execute(&mut *connection)
    .await?;

    // Add partial unique index for active download operations
    ormlite::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_download_queue_active_ops ON download_queue(cloud_map_id) 
         WHERE status IN ('pending', 'in_progress');"
    )
    .execute(&mut *connection)
    .await?;

    // Upload queue table
    ormlite::query(
        "CREATE TABLE IF NOT EXISTS upload_queue (
            id TEXT PRIMARY KEY NOT NULL,
            cloud_map_id TEXT NOT NULL,
            provider_type TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL,
            error_message TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            attempts INTEGER NOT NULL DEFAULT 0
        );"
    )
    .execute(&mut *connection)
    .await?;

    // Create indexes for upload queue
    ormlite::query(
        "CREATE INDEX IF NOT EXISTS idx_upload_queue_cloud_map_id ON upload_queue(cloud_map_id);
         CREATE INDEX IF NOT EXISTS idx_upload_queue_status ON upload_queue(status);
         CREATE INDEX IF NOT EXISTS idx_upload_queue_map_status ON upload_queue(cloud_map_id, status);"
    )
    .execute(&mut *connection)
    .await?;

    // Add partial unique index for active upload operations
    ormlite::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_upload_queue_active_ops ON upload_queue(cloud_map_id) 
         WHERE status IN ('pending', 'in_progress');"
    )
    .execute(&mut *connection)
    .await?;

    // Create unified tracks view
    ormlite::query(
        "CREATE VIEW IF NOT EXISTS unified_tracks AS
        WITH base_tracks AS (
            -- Get all tracks by path
            SELECT 
                relative_path,
                'path' as join_type
            FROM (
                SELECT ctm.relative_path 
                FROM cloud_maps ctm
                UNION 
                SELECT SUBSTR(t.path, LENGTH(cmf.local_folder_path) + 1) as relative_path
                FROM tracks t
                JOIN cloud_music_folders cmf 
                WHERE t.path LIKE cmf.local_folder_path || '%'
            )
        )
        SELECT 
            -- Track identifiers
            t.id as local_track_id,
            ct.id as cloud_track_id,
            ctm.id as cloud_map_id,
            cmf.id as cloud_folder_id,

            -- Paths and locations
            t.path as local_path,
            ctm.relative_path as cloud_relative_path,
            cmf.cloud_folder_path as cloud_folder_path,
            cmf.local_folder_path as cloud_local_folder_path,
            cmf.provider_type as cloud_provider_type,
            ctm.cloud_file_id,

            -- Metadata (preferring local over cloud)
            COALESCE(t.title, ct.tags->>'$.title', ct.file_name) as title,
            COALESCE(t.album, ct.tags->>'$.album', 'Unknown') as album,
            COALESCE(
                json(t.artists),
                ct.tags->>'$.artists',
                json_array('Unknown Artist')
            ) as artists,
            COALESCE(
                json(t.genres),
                ct.tags->>'$.genres',
                json_array()
            ) as genres,
            COALESCE(t.year, CAST(ct.tags->>'$.year' AS INTEGER)) as year,
            COALESCE(t.duration, CAST(ct.tags->>'$.duration' AS INTEGER), 0) as duration,
            COALESCE(t.track_no, CAST(ct.tags->>'$.track_no' AS INTEGER)) as track_no,
            COALESCE(t.track_of, CAST(ct.tags->>'$.track_of' AS INTEGER)) as track_of,
            COALESCE(t.disk_no, CAST(ct.tags->>'$.disk_no' AS INTEGER)) as disk_no,
            COALESCE(t.disk_of, CAST(ct.tags->>'$.disk_of' AS INTEGER)) as disk_of,

            -- Location and sync state
            CASE 
                WHEN t.id IS NOT NULL AND ct.id IS NOT NULL THEN 'both'
                WHEN t.id IS NOT NULL THEN 'local'
                ELSE 'cloud'
            END as location_type,
            ct.updated_at as cloud_updated_at

        FROM base_tracks
        -- Join with tracks based on path
        LEFT JOIN cloud_maps ctm ON 
            base_tracks.relative_path = ctm.relative_path
        LEFT JOIN cloud_tracks ct ON 
            ct.id = ctm.cloud_track_id
        LEFT JOIN cloud_music_folders cmf ON 
            ctm.cloud_music_folder_id = cmf.id
        LEFT JOIN tracks t ON 
            t.path = cmf.local_folder_path || base_tracks.relative_path;"
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}
