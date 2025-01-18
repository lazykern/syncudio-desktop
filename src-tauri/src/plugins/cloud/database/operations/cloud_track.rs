use ormlite::Model;

use crate::libs::database::core::DB;
use crate::libs::error::AnyResult;
use crate::plugins::cloud::models::{
    cloud_track::CloudTrack,
    query_models::TrackWithMapRow,
};

impl DB {
    // Basic CRUD Operations
    pub async fn get_cloud_track(&mut self, id: &str) -> AnyResult<Option<CloudTrack>> {
        let track = CloudTrack::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(track)
    }

    pub async fn get_cloud_track_by_hash(&mut self, hash: &str) -> AnyResult<Option<CloudTrack>> {
        let track = CloudTrack::select()
            .where_bind("blake3_hash = ?", hash)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(track)
    }

    pub async fn get_cloud_track_by_cloud_id(&mut self, cloud_id: &str) -> AnyResult<Option<CloudTrack>> {
        let track = CloudTrack::select()
            .where_bind("cloud_file_id = ?", cloud_id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(track)
    }

    pub async fn get_cloud_tracks(&mut self) -> AnyResult<Vec<CloudTrack>> {
        let tracks = CloudTrack::select()
            .fetch_all(&mut self.connection)
            .await?;
        Ok(tracks)
    }

    pub async fn save_cloud_track(&mut self, track: CloudTrack) -> AnyResult<CloudTrack> {
        let saved = track.insert(&mut self.connection).await?;
        Ok(saved)
    }

    pub async fn update_cloud_track(&mut self, track: CloudTrack) -> AnyResult<CloudTrack> {
        let updated = track.update_all_fields(&mut self.connection).await?;
        Ok(updated)
    }

    pub async fn delete_cloud_track(&mut self, id: &str) -> AnyResult<()> {
        if let Some(track) = CloudTrack::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await? {
            track.delete(&mut self.connection).await?;
        }
        Ok(())
    }

    // Complex Query Operations
    pub async fn get_cloud_track_by_hash_or_cloud_id(
        &mut self,
        hash: Option<&str>,
        cloud_id: Option<&str>
    ) -> AnyResult<Option<CloudTrack>> {
        let track = CloudTrack::select()
            .where_("(blake3_hash IS NOT NULL AND blake3_hash = ?) OR (cloud_file_id IS NOT NULL AND cloud_file_id = ?)")
            .bind(hash)
            .bind(cloud_id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(track)
    }

    pub async fn get_cloud_tracks_with_maps(&mut self, folder_id: &str) -> AnyResult<Vec<TrackWithMapRow>> {
        let tracks = ormlite::query_as(r#"
            SELECT 
                t.id, t.blake3_hash, t.cloud_file_id, t.file_name, t.updated_at, t.tags,
                m.id as map_id, m.relative_path, m.cloud_folder_id
            FROM cloud_tracks t
            INNER JOIN cloud_track_maps m ON t.id = m.cloud_track_id
            WHERE m.cloud_folder_id = ?
        "#)
        .bind(folder_id)
        .fetch_all(&mut self.connection)
        .await?;
        Ok(tracks)
    }

    pub async fn cleanup_orphaned_tracks(&mut self) -> AnyResult<()> {
        CloudTrack::query(r#"
            DELETE FROM cloud_tracks 
            WHERE blake3_hash IS NULL 
            AND cloud_file_id IS NULL
        "#)
        .fetch_all(&mut self.connection)
        .await?;
        Ok(())
    }
} 