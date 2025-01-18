use cloud_folder::CloudFolder;
use ormlite::Model;

use crate::libs::database::core::DB;
use crate::libs::error::AnyResult;
use crate::plugins::cloud::models::*;

impl DB {
    // Folder operations
    pub async fn get_cloud_folder(&mut self, id: &str) -> AnyResult<Option<CloudFolder>> {
        let folder = CloudFolder::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(folder)
    }
    
    pub async fn get_cloud_folders(&mut self) -> AnyResult<Vec<CloudFolder>> {
        let folders = CloudFolder::select()
            .fetch_all(&mut self.connection)
            .await?;
        Ok(folders)
    }

    pub async fn get_cloud_folders_by_provider(
        &mut self,
        provider_type: &str,
    ) -> AnyResult<Vec<CloudFolder>> {
        let folders = CloudFolder::select()
            .where_bind("provider_type = ?", provider_type)
            .fetch_all(&mut self.connection)
            .await?;
        Ok(folders)
    }

    pub async fn get_cloud_folder_by_local_path(
        &mut self,
        local_path: &str,
    ) -> AnyResult<Option<CloudFolder>> {
        let folder = CloudFolder::select()
            .where_bind("local_folder_path = ?", local_path)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(folder)
    }

    pub async fn save_cloud_folder(&mut self, folder: CloudFolder) -> AnyResult<CloudFolder> {
        let saved = folder.insert(&mut self.connection).await?;
        Ok(saved)
    }

    pub async fn update_cloud_folder(&mut self, folder: CloudFolder) -> AnyResult<CloudFolder> {
        let updated = folder.update_all_fields(&mut self.connection).await?;
        Ok(updated)
    }

    pub async fn delete_cloud_folder(&mut self, id: &str) -> AnyResult<()> {
        if let Some(folder) = CloudFolder::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await? {
            folder.delete(&mut self.connection).await?;
        }
        Ok(())
    }
}
