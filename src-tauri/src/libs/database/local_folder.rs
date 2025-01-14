use ormlite::model::ModelBuilder;
use ormlite::Model;

use crate::libs::error::AnyResult;
use crate::libs::local_folder::LocalFolder;

use super::core::DB;

impl DB {
    /// Get all local folders from the database
    pub async fn get_all_local_folders(&mut self) -> AnyResult<Vec<LocalFolder>> {
        let local_folders = LocalFolder::select()
            .fetch_all(&mut self.connection)
            .await?;
        Ok(local_folders)
    }

    /// Get a single local folder by path
    pub async fn get_local_folder(&mut self, path: &str) -> AnyResult<Option<LocalFolder>> {
        let local_folder = LocalFolder::select()
            .where_bind("path = ?", path)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(local_folder)
    }

    /// Create a new local folder
    pub async fn create_local_folder(&mut self, path: String) -> AnyResult<LocalFolder> {
        let local_folder = LocalFolder {
            path,
        };

        let local_folder = local_folder.insert(&mut self.connection).await?;
        Ok(local_folder)
    }

    /// Delete a local folder by ID
    pub async fn delete_local_folder(&mut self, path: &str) -> AnyResult<()> {
        let local_folder = LocalFolder::select()
            .where_bind("path = ?", path)
            .fetch_one(&mut self.connection)
            .await?;

        local_folder.delete(&mut self.connection).await?;
        Ok(())
    }
} 