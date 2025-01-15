use ormlite::sqlite::SqliteConnection;
use crate::libs::error::AnyResult;


pub async fn create_tables(connection: &mut SqliteConnection) -> AnyResult<()> {

    // Cloud folder mappings
    ormlite::query(
        "CREATE TABLE IF NOT EXISTS cloud_folders (
            id TEXT PRIMARY KEY NOT NULL,
            provider_type TEXT NOT NULL,
            cloud_folder_id TEXT NOT NULL,
            cloud_folder_name TEXT NOT NULL,
            local_folder_path TEXT NOT NULL UNIQUE
        );",
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
} 