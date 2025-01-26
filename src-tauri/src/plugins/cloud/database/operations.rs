use core::str;

use cloud_music_folder::CloudMusicFolder;
use ormlite::Model;

use crate::libs::database::core::DB;
use crate::libs::error::AnyResult;
use crate::plugins::cloud::models::*;

impl DB {
    // Folder operations
    pub async fn get_cloud_folder(&mut self, id: &str) -> AnyResult<Option<CloudMusicFolder>> {
        let folder = CloudMusicFolder::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(folder)
    }
    
    pub async fn get_cloud_music_folders(&mut self) -> AnyResult<Vec<CloudMusicFolder>> {
        let folders = CloudMusicFolder::select()
            .fetch_all(&mut self.connection)
            .await?;
        Ok(folders)
    }

    pub async fn get_cloud_music_folders_by_provider(
        &mut self,
        provider_type: &str,
    ) -> AnyResult<Vec<CloudMusicFolder>> {
        let folders = CloudMusicFolder::select()
            .where_bind("provider_type = ?", provider_type)
            .fetch_all(&mut self.connection)
            .await?;
        Ok(folders)
    }

    pub async fn get_cloud_folder_by_local_path(
        &mut self,
        local_path: &str,
    ) -> AnyResult<Option<CloudMusicFolder>> {
        let folder = CloudMusicFolder::select()
            .where_bind("local_folder_path = ?", local_path)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(folder)
    }

    pub async fn save_cloud_folder(&mut self, folder: CloudMusicFolder) -> AnyResult<CloudMusicFolder> {
        let saved = folder.insert(&mut self.connection).await?;
        Ok(saved)
    }

    pub async fn update_cloud_folder(&mut self, folder: CloudMusicFolder) -> AnyResult<CloudMusicFolder> {
        let updated = folder.update_all_fields(&mut self.connection).await?;
        Ok(updated)
    }

    pub async fn delete_cloud_folder(&mut self, id: &str) -> AnyResult<()> {
        if let Some(folder) = CloudMusicFolder::select()
            .where_bind("id = ?", id)
            .fetch_optional(&mut self.connection)
            .await? {
            folder.delete(&mut self.connection).await?;
        }
        Ok(())
    }

    pub async fn get_unified_tracks(&mut self) -> AnyResult<Vec<UnifiedTrack>> {
        let tracks = ormlite::query_as("SELECT * FROM unified_tracks;")
            .fetch_all(&mut self.connection)
            .await?;
        Ok(tracks)
    }

    pub async fn get_unified_tracks_by_ids(&mut self, ids: &[String]) -> AnyResult<Vec<UnifiedTrack>> {
        let mut query = "SELECT * FROM unified_tracks WHERE local_track_id IN (".to_string();

        query.push_str(&std::iter::repeat("?").take(ids.len()).collect::<Vec<_>>().join(","));
        query.push_str(") OR cloud_track_id IN (");
        query.push_str(&std::iter::repeat("?").take(ids.len()).collect::<Vec<_>>().join(","));
        query.push_str(")");

        let mut q_builder = ormlite::query_as(&query);

        for id in ids {
            q_builder = q_builder.bind(id);
        }

        for id in ids {
            q_builder = q_builder.bind(id);
        }

        let tracks = q_builder.fetch_all(&mut self.connection).await?;
        Ok(tracks)
    }

    pub async fn get_unified_tracks_by_location(&mut self, location_type: &str) -> AnyResult<Vec<UnifiedTrack>> {
        let tracks = ormlite::query_as("SELECT * FROM unified_tracks WHERE location_type = ?")
            .bind(location_type)
            .fetch_all(&mut self.connection)
            .await?;
        Ok(tracks)
    }

    pub async fn get_unified_tracks_by_folder(&mut self, folder_id: &str) -> AnyResult<Vec<UnifiedTrack>> {
        let tracks = ormlite::query_as("SELECT * FROM unified_tracks WHERE cloud_folder_id = ?")
            .bind(folder_id)
            .fetch_all(&mut self.connection)
            .await?;
        Ok(tracks)
    }

    pub async fn get_unified_tracks_by_provider(&mut self, provider_type: &str) -> AnyResult<Vec<UnifiedTrack>> {
        let tracks = ormlite::query_as("SELECT * FROM unified_tracks WHERE cloud_provider_type = ?")
            .bind(provider_type)
            .fetch_all(&mut self.connection)
            .await?;
        Ok(tracks)
    }

    pub async fn get_unified_track(&mut self, id: &str) -> AnyResult<Option<UnifiedTrack>> {
        let tracks = ormlite::query_as("SELECT * FROM unified_tracks WHERE local_track_id = ? OR cloud_track_id = ?")
            .bind(id)
            .bind(id)
            .fetch_optional(&mut self.connection)
            .await?;
        Ok(tracks)
    }

    /// Get all cloud tracks with their mapping and folder information
    pub async fn get_cloud_tracks_full(&mut self) -> AnyResult<Vec<CloudTrackFullDTO>> {
        let tracks = ormlite::query_as(
            r#"
            SELECT 
                -- CloudTrack fields
                ct.id as track_id,
                ct.blake3_hash,
                ct.file_name,
                ct.updated_at as track_updated_at,
                ct.tags,

                -- CloudTrackMap fields
                cm.id as map_id,
                cm.cloud_file_id,
                cm.relative_path,

                -- CloudMusicFolder fields
                cmf.id as folder_id,
                cmf.provider_type,
                cmf.cloud_folder_id,
                cmf.cloud_folder_path,
                cmf.local_folder_path

            FROM cloud_tracks ct
            INNER JOIN cloud_maps cm ON ct.id = cm.cloud_track_id
            INNER JOIN cloud_music_folders cmf ON cm.cloud_music_folder_id = cmf.id
            "#,
        )
        .fetch_all(&mut self.connection)
        .await?;

        Ok(tracks)
    }

    /// Get cloud tracks for a specific folder
    pub async fn get_cloud_tracks_full_by_folder(&mut self, folder_id: &str) -> AnyResult<Vec<CloudTrackFullDTO>> {
        let tracks = ormlite::query_as(
            r#"
            SELECT 
                -- CloudTrack fields
                ct.id as track_id,
                ct.blake3_hash,
                ct.file_name,
                ct.updated_at as track_updated_at,
                ct.tags,

                -- CloudTrackMap fields
                cm.id as map_id,
                cm.cloud_file_id,
                cm.relative_path,

                -- CloudMusicFolder fields
                cmf.id as folder_id,
                cmf.provider_type,
                cmf.cloud_folder_id,
                cmf.cloud_folder_path,
                cmf.local_folder_path

            FROM cloud_tracks ct
            INNER JOIN cloud_maps cm ON ct.id = cm.cloud_track_id
            INNER JOIN cloud_music_folders cmf ON cm.cloud_music_folder_id = cmf.id
            WHERE cmf.id = ?
            "#,
        )
        .bind(folder_id)
        .fetch_all(&mut self.connection)
        .await?;

        Ok(tracks)
    }

    /// Get cloud tracks for a specific provider
    pub async fn get_cloud_tracks_full_by_provider(&mut self, provider_type: &str) -> AnyResult<Vec<CloudTrackFullDTO>> {
        let tracks = ormlite::query_as(
            r#"
            SELECT 
                -- CloudTrack fields
                ct.id as track_id,
                ct.blake3_hash,
                ct.file_name,
                ct.updated_at as track_updated_at,
                ct.tags,

                -- CloudTrackMap fields
                cm.id as map_id,
                cm.cloud_file_id,
                cm.relative_path,

                -- CloudMusicFolder fields
                cmf.id as folder_id,
                cmf.provider_type,
                cmf.cloud_folder_id,
                cmf.cloud_folder_path,
                cmf.local_folder_path

            FROM cloud_tracks ct
            INNER JOIN cloud_maps cm ON ct.id = cm.cloud_track_id
            INNER JOIN cloud_music_folders cmf ON cm.cloud_music_folder_id = cmf.id
            WHERE cmf.provider_type = ?
            "#,
        )
        .bind(provider_type)
        .fetch_all(&mut self.connection)
        .await?;

        Ok(tracks)
    }

    /// Get a single cloud track by its ID
    pub async fn get_cloud_track_full(&mut self, track_id: &str) -> AnyResult<Option<CloudTrackFullDTO>> {
        let track = ormlite::query_as(
            r#"
            SELECT 
                -- CloudTrack fields
                ct.id as track_id,
                ct.blake3_hash,
                ct.file_name,
                ct.updated_at as track_updated_at,
                ct.tags,

                -- CloudTrackMap fields
                cm.id as map_id,
                cm.cloud_file_id,
                cm.relative_path,

                -- CloudMusicFolder fields
                cmf.id as folder_id,
                cmf.provider_type,
                cmf.cloud_folder_id,
                cmf.cloud_folder_path,
                cmf.local_folder_path

            FROM cloud_tracks ct
            INNER JOIN cloud_maps cm ON ct.id = cm.cloud_track_id
            INNER JOIN cloud_music_folders cmf ON cm.cloud_music_folder_id = cmf.id
            WHERE ct.id = ?
            "#,
        )
        .bind(track_id)
        .fetch_optional(&mut self.connection)
        .await?;

        Ok(track)
    }
}
