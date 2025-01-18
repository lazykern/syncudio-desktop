use ormlite::Model;

use crate::libs::database::core::DB;
use crate::libs::error::AnyResult;
use crate::plugins::cloud::models::cloud_track::CloudTrackMap;

impl DB {
    // Basic CRUD Operations
    pub async fn get_cloud_track_map(&mut self, id: &str) -> AnyResult<Option<CloudTrackMap>> {
        let map = CloudTrackMap::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(map)
    }

    pub async fn get_cloud_track_map_by_track_id(&mut self, track_id: &str) -> AnyResult<Option<CloudTrackMap>> {
        let map = CloudTrackMap::select()
            .where_bind("cloud_track_id = ?", track_id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(map)
    }

    pub async fn get_cloud_track_map_by_track_id_and_folder_id(
        &mut self,
        track_id: &str,
        folder_id: &str
    ) -> AnyResult<Option<CloudTrackMap>> {
        let map = CloudTrackMap::select()
            .where_("cloud_track_id = ? AND cloud_folder_id = ?")
            .bind(track_id)
            .bind(folder_id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(map)
    }

    pub async fn get_cloud_track_maps_by_folder_id(&mut self, folder_id: &str) -> AnyResult<Vec<CloudTrackMap>> {
        let maps = CloudTrackMap::select()
            .where_bind("cloud_folder_id = ?", folder_id)
            .fetch_all(&mut self.connection)
            .await?;
        Ok(maps)
    }


    pub async fn delete_cloud_track_map(&mut self, id: &str) -> AnyResult<()> {
        if let Some(map) = CloudTrackMap::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await? {
            map.delete(&mut self.connection).await?;
        }
        Ok(())
    }

    // Complex Query Operations
    pub async fn get_cloud_track_map_by_track_and_folder(
        &mut self,
        track_id: &str,
        folder_id: &str
    ) -> AnyResult<Option<CloudTrackMap>> {
        let map = CloudTrackMap::select()
            .where_("cloud_track_id = ? AND cloud_folder_id = ?")
            .bind(track_id)
            .bind(folder_id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(map)
    }

    pub async fn get_cloud_track_maps_by_track_ids(&mut self, track_ids: &[String]) -> AnyResult<Vec<CloudTrackMap>> {
        let placeholders = vec!["?"; track_ids.len()].join(",");
        let query = format!("SELECT * FROM cloud_track_maps WHERE cloud_track_id IN ({})", placeholders);
        
        let mut query_builder = ormlite::query_as(&query);
        for id in track_ids {
            query_builder = query_builder.bind(id);
        }
        
        let maps = query_builder.fetch_all(&mut self.connection).await?;
        Ok(maps)
    }

    pub async fn delete_cloud_track_maps_by_folder_id(&mut self, folder_id: &str) -> AnyResult<()> {
        CloudTrackMap::query(r#"
            DELETE FROM cloud_track_maps 
            WHERE cloud_folder_id = ?
        "#)
        .bind(folder_id)
        .fetch_all(&mut self.connection)
        .await?;
        Ok(())
    }
} 