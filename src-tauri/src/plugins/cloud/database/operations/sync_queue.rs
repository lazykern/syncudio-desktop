use ormlite::Model;

use crate::libs::database::core::DB;
use crate::libs::error::AnyResult;
use crate::plugins::cloud::models::{
    sync_queue::{UploadQueueItem, DownloadQueueItem},
    query_models::{QueueOperationRow, QueueStatsRow},
};

impl DB {
    // Upload Queue Operations
    pub async fn get_upload_queue_item(&mut self, id: &str) -> AnyResult<Option<UploadQueueItem>> {
        let item = UploadQueueItem::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(item)
    }

    pub async fn get_upload_queue_items_by_folder_id(&mut self, folder_id: &str) -> AnyResult<Vec<UploadQueueItem>> {
        let items = UploadQueueItem::query(r#"
            SELECT * FROM upload_queue 
            WHERE cloud_track_map_id IN (
                SELECT id FROM cloud_track_maps WHERE cloud_folder_id = ?
            ) ORDER BY created_at ASC
        "#)
        .bind(folder_id)
        .fetch_all(&mut self.connection)
        .await?;
        Ok(items)
    }

    pub async fn get_upload_queue_items(&mut self) -> AnyResult<Vec<UploadQueueItem>> {
        let items = UploadQueueItem::query("SELECT * FROM upload_queue ORDER BY created_at ASC")
            .fetch_all(&mut self.connection)
            .await?;
        Ok(items)
    }

    pub async fn save_upload_queue_item(&mut self, item: UploadQueueItem) -> AnyResult<UploadQueueItem> {
        let saved = item.insert(&mut self.connection).await?;
        Ok(saved)
    }

    pub async fn update_upload_queue_item(&mut self, item: UploadQueueItem) -> AnyResult<UploadQueueItem> {
        let updated = item.update_all_fields(&mut self.connection).await?;
        Ok(updated)
    }

    pub async fn delete_upload_queue_item(&mut self, id: &str) -> AnyResult<()> {
        if let Some(item) = UploadQueueItem::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await? {
            item.delete(&mut self.connection).await?;
        }
        Ok(())
    }

    pub async fn get_active_upload_queue_item(&mut self, map_id: &str) -> AnyResult<Option<UploadQueueItem>> {
        let item = UploadQueueItem::select()
            .where_("cloud_track_map_id = ? AND (status = 'pending' OR status = 'in_progress')")
            .bind(map_id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(item)
    }

    // Download Queue Operations
    pub async fn get_download_queue_item(&mut self, id: &str) -> AnyResult<Option<DownloadQueueItem>> {
        let item = DownloadQueueItem::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(item)
    }

    pub async fn get_download_queue_items_by_folder_id(&mut self, folder_id: &str) -> AnyResult<Vec<DownloadQueueItem>> {
        let items = DownloadQueueItem::query(r#"
            SELECT * FROM download_queue 
            WHERE cloud_track_map_id IN (
                SELECT id FROM cloud_track_maps WHERE cloud_folder_id = ?
            ) ORDER BY created_at ASC
        "#)
        .bind(folder_id)
        .fetch_all(&mut self.connection)
        .await?;
        Ok(items)
    }

    pub async fn get_download_queue_items(&mut self) -> AnyResult<Vec<DownloadQueueItem>> {
        let items = DownloadQueueItem::query("SELECT * FROM download_queue ORDER BY created_at ASC")
            .fetch_all(&mut self.connection)
            .await?;
        Ok(items)
    }

    pub async fn save_download_queue_item(&mut self, item: DownloadQueueItem) -> AnyResult<DownloadQueueItem> {
        let saved = item.insert(&mut self.connection).await?;
        Ok(saved)
    }

    pub async fn update_download_queue_item(&mut self, item: DownloadQueueItem) -> AnyResult<DownloadQueueItem> {
        let updated = item.update_all_fields(&mut self.connection).await?;
        Ok(updated)
    }

    pub async fn delete_download_queue_item(&mut self, id: &str) -> AnyResult<()> {
        if let Some(item) = DownloadQueueItem::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await? {
            item.delete(&mut self.connection).await?;
        }
        Ok(())
    }

    pub async fn get_active_download_queue_item(&mut self, map_id: &str) -> AnyResult<Option<DownloadQueueItem>> {
        let item = DownloadQueueItem::select()
            .where_("cloud_track_map_id = ? AND (status = 'pending' OR status = 'in_progress')")
            .bind(map_id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(item)
    }

    pub async fn get_queue_items_by_track_ids(&mut self, track_ids: &[String]) -> AnyResult<Vec<UploadQueueItem>> {
        let placeholders = vec!["?"; track_ids.len()].join(",");
        let query = format!(
            "SELECT * FROM upload_queue WHERE cloud_track_map_id IN (
                SELECT id FROM cloud_track_maps WHERE cloud_track_id IN ({})
            )",
            placeholders
        );
        
        let mut query_builder = ormlite::query_as(&query);
        for id in track_ids {
            query_builder = query_builder.bind(id);
        }
        
        let items = query_builder.fetch_all(&mut self.connection).await?;
        Ok(items)
    }

    // Combined Queue Operations
    pub async fn get_active_operations(&mut self, folder_id: &str) -> AnyResult<Vec<QueueOperationRow>> {
        let operations = ormlite::query_as(r#"
            SELECT 
                'upload' as queue_type,
                u.status,
                u.cloud_track_map_id
            FROM upload_queue u
            INNER JOIN cloud_track_maps m ON u.cloud_track_map_id = m.id
            WHERE m.cloud_folder_id = ? 
                AND (u.status = 'pending' OR u.status = 'in_progress')
            UNION ALL
            SELECT 
                'download' as queue_type,
                d.status,
                d.cloud_track_map_id
            FROM download_queue d
            INNER JOIN cloud_track_maps m ON d.cloud_track_map_id = m.id
            WHERE m.cloud_folder_id = ?
                AND (d.status = 'pending' OR d.status = 'in_progress')
        "#)
        .bind(folder_id)
        .bind(folder_id)
        .fetch_all(&mut self.connection)
        .await?;
        Ok(operations)
    }

    pub async fn get_queue_stats(&mut self, folder_id: Option<&str>) -> AnyResult<Vec<QueueStatsRow>> {
        let stats = if let Some(folder_id) = folder_id {
            ormlite::query_as(r#"
                SELECT status, COUNT(*) as count
                FROM (
                    SELECT status FROM upload_queue u
                    INNER JOIN cloud_track_maps m ON u.cloud_track_map_id = m.id
                    WHERE m.cloud_folder_id = ?
                    UNION ALL
                    SELECT status FROM download_queue d
                    INNER JOIN cloud_track_maps m ON d.cloud_track_map_id = m.id
                    WHERE m.cloud_folder_id = ?
                ) combined
                GROUP BY status
            "#)
            .bind(folder_id)
            .bind(folder_id)
            .fetch_all(&mut self.connection)
            .await?
        } else {
            ormlite::query_as(r#"
                SELECT status, COUNT(*) as count
                FROM (
                    SELECT status FROM upload_queue
                    UNION ALL
                    SELECT status FROM download_queue
                ) combined
                GROUP BY status
            "#)
            .fetch_all(&mut self.connection)
            .await?
        };
        Ok(stats)
    }
} 