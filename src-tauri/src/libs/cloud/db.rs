use ormlite::{sqlite::SqliteConnection, Model};
use uuid::Uuid;

use super::models::*;
use crate::libs::error::AnyResult;

pub struct CloudDB {
    pub connection: SqliteConnection,
}

impl CloudDB {
    // Provider operations
    pub async fn get_provider(&mut self, id: &str) -> AnyResult<Option<CloudProvider>> {
        let provider = CloudProvider::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;
        Ok(Some(provider))
    }

    pub async fn get_provider_by_type(&mut self, provider_type: &str) -> AnyResult<Option<CloudProvider>> {
        let provider = CloudProvider::select()
            .where_bind("provider_type = ?", provider_type)
            .fetch_one(&mut self.connection)
            .await?;
        Ok(Some(provider))
    }

    pub async fn save_provider(&mut self, provider: CloudProvider) -> AnyResult<CloudProvider> {
        let saved = provider.insert(&mut self.connection).await?;
        Ok(saved)
    }

    pub async fn update_provider(&mut self, provider: CloudProvider) -> AnyResult<CloudProvider> {
        let updated = provider.update_all_fields(&mut self.connection).await?;
        Ok(updated)
    }

    pub async fn delete_provider(&mut self, id: &str) -> AnyResult<()> {
        let provider = CloudProvider::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;
        provider.delete(&mut self.connection).await?;
        Ok(())
    }

    // Folder operations
    pub async fn get_folder(&mut self, id: &str) -> AnyResult<Option<CloudFolder>> {
        let folder = CloudFolder::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;
        Ok(Some(folder))
    }

    pub async fn get_folders_by_provider(&mut self, provider_id: &str) -> AnyResult<Vec<CloudFolder>> {
        let folders = CloudFolder::select()
            .where_bind("provider_id = ?", provider_id)
            .fetch_all(&mut self.connection)
            .await?;
        Ok(folders)
    }

    pub async fn get_folder_by_local_path(&mut self, local_path: &str) -> AnyResult<Option<CloudFolder>> {
        let folder = CloudFolder::select()
            .where_bind("local_folder_path = ?", local_path)
            .fetch_one(&mut self.connection)
            .await?;
        Ok(Some(folder))
    }

    pub async fn save_folder(&mut self, folder: CloudFolder) -> AnyResult<CloudFolder> {
        let saved = folder.insert(&mut self.connection).await?;
        Ok(saved)
    }

    pub async fn update_folder(&mut self, folder: CloudFolder) -> AnyResult<CloudFolder> {
        let updated = folder.update_all_fields(&mut self.connection).await?;
        Ok(updated)
    }

    pub async fn delete_folder(&mut self, id: &str) -> AnyResult<()> {
        let folder = CloudFolder::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;
        folder.delete(&mut self.connection).await?;
        Ok(())
    }

    // Sync operations
    pub async fn get_sync(&mut self, id: &str) -> AnyResult<Option<CloudSync>> {
        let sync = CloudSync::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;
        Ok(Some(sync))
    }

    pub async fn get_sync_by_item(&mut self, item_id: &str, item_type: &str) -> AnyResult<Option<CloudSync>> {
        let sync = CloudSync::select()
            .where_("item_id = ? AND item_type = ?")
            .bind(item_id)
            .bind(item_type)
            .fetch_one(&mut self.connection)
            .await?;
        Ok(Some(sync))
    }

    pub async fn get_syncs_by_folder(&mut self, folder_id: &str) -> AnyResult<Vec<CloudSync>> {
        let syncs = CloudSync::select()
            .where_bind("folder_id = ?", folder_id)
            .fetch_all(&mut self.connection)
            .await?;
        Ok(syncs)
    }

    pub async fn get_syncs_by_status(&mut self, status: &str) -> AnyResult<Vec<CloudSync>> {
        let syncs = CloudSync::select()
            .where_bind("sync_status = ?", status)
            .fetch_all(&mut self.connection)
            .await?;
        Ok(syncs)
    }

    pub async fn get_pending_syncs(&mut self) -> AnyResult<Vec<CloudSync>> {
        let syncs = CloudSync::select()
            .where_("sync_status IN (?, ?)")
            .bind(SYNC_STATUS_PENDING_UPLOAD)
            .bind(SYNC_STATUS_PENDING_DOWNLOAD)
            .fetch_all(&mut self.connection)
            .await?;
        Ok(syncs)
    }

    pub async fn save_sync(&mut self, sync: CloudSync) -> AnyResult<CloudSync> {
        let saved = sync.insert(&mut self.connection).await?;
        Ok(saved)
    }

    pub async fn update_sync(&mut self, sync: CloudSync) -> AnyResult<CloudSync> {
        let updated = sync.update_all_fields(&mut self.connection).await?;
        Ok(updated)
    }

    pub async fn delete_sync(&mut self, id: &str) -> AnyResult<()> {
        let sync = CloudSync::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;
        sync.delete(&mut self.connection).await?;
        Ok(())
    }

    pub async fn delete_syncs_by_folder(&mut self, folder_id: &str) -> AnyResult<()> {
        let syncs = CloudSync::select()
            .where_bind("folder_id = ?", folder_id)
            .fetch_all(&mut self.connection)
            .await?;
        for sync in syncs {
            sync.delete(&mut self.connection).await?;
        }
        Ok(())
    }

    // Helper methods
    pub async fn create_sync_for_track(
        &mut self,
        provider_id: &str,
        folder_id: &str,
        track_id: &str,
        cloud_file_id: &str,
        cloud_file_name: &str,
        local_path: &str,
    ) -> AnyResult<CloudSync> {
        let sync = CloudSync::new(
            Uuid::new_v4().to_string(),
            provider_id.to_string(),
            folder_id.to_string(),
            track_id.to_string(),
            ITEM_TYPE_TRACK.to_string(),
            cloud_file_id.to_string(),
            cloud_file_name.to_string(),
            local_path.to_string(),
            SYNC_STATUS_SYNCED.to_string(),
        );
        self.save_sync(sync).await
    }

    pub async fn create_sync_for_playlist(
        &mut self,
        provider_id: &str,
        folder_id: &str,
        playlist_id: &str,
        cloud_file_id: &str,
        cloud_file_name: &str,
        local_path: &str,
    ) -> AnyResult<CloudSync> {
        let sync = CloudSync::new(
            Uuid::new_v4().to_string(),
            provider_id.to_string(),
            folder_id.to_string(),
            playlist_id.to_string(),
            ITEM_TYPE_PLAYLIST.to_string(),
            cloud_file_id.to_string(),
            cloud_file_name.to_string(),
            local_path.to_string(),
            SYNC_STATUS_SYNCED.to_string(),
        );
        self.save_sync(sync).await
    }
} 