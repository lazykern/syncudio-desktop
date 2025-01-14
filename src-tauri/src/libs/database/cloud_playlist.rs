use ormlite::model::ModelBuilder;
use ormlite::Model;

use crate::libs::cloud_playlist::CloudPlaylist;
use crate::libs::error::AnyResult;
use crate::libs::utils::TimeLogger;

use super::core::DB;

impl DB {
    /// Get all cloud playlists from the database
    pub async fn get_all_cloud_playlists(&mut self) -> AnyResult<Vec<CloudPlaylist>> {
        let timer = TimeLogger::new("Retrieved and decoded cloud playlists".into());
        let mut playlists = CloudPlaylist::select()
            .order_asc("name")
            .fetch_all(&mut self.connection)
            .await?;

        // Ensure the playlists are sorted alphabetically (case-insensitive) for better UX
        playlists.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        timer.complete();
        Ok(playlists)
    }

    /// Get a single cloud playlist by ID
    pub async fn get_cloud_playlist(&mut self, playlist_id: &String) -> AnyResult<Option<CloudPlaylist>> {
        let playlist = CloudPlaylist::select()
            .where_bind("id = ?", playlist_id)
            .fetch_one(&mut self.connection)
            .await?;

        Ok(Some(playlist))
    }

    /// Create a cloud playlist with the given details
    pub async fn create_cloud_playlist(
        &mut self,
        name: String,
        description: Option<String>,
        tracks: Vec<String>,
    ) -> AnyResult<CloudPlaylist> {
        let playlist = CloudPlaylist::new(name);
        let playlist = CloudPlaylist {
            description,
            tracks,
            ..playlist
        };

        let playlist = playlist.insert(&mut self.connection).await?;
        Ok(playlist)
    }

    /// Update a cloud playlist's tracks
    pub async fn set_cloud_playlist_tracks(
        &mut self,
        id: &String,
        tracks: Vec<String>,
    ) -> AnyResult<CloudPlaylist> {
        let playlist = CloudPlaylist::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;

        let updated_playlist = playlist
            .update_partial()
            .tracks(tracks)
            .updated_at(chrono::Utc::now().timestamp())
            .update(&mut self.connection)
            .await?;

        Ok(updated_playlist)
    }

    /// Update a cloud playlist's name
    pub async fn rename_cloud_playlist(&mut self, id: &String, name: String) -> AnyResult<CloudPlaylist> {
        let playlist = CloudPlaylist::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;

        let updated_playlist = playlist
            .update_partial()
            .name(name)
            .updated_at(chrono::Utc::now().timestamp())
            .update(&mut self.connection)
            .await?;

        Ok(updated_playlist)
    }

    /// Update a cloud playlist's description
    pub async fn update_cloud_playlist_description(
        &mut self,
        id: &String,
        description: Option<String>,
    ) -> AnyResult<CloudPlaylist> {
        let playlist = CloudPlaylist::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;

        let updated_playlist = playlist
            .update_partial()
            .description(description)
            .updated_at(chrono::Utc::now().timestamp())
            .update(&mut self.connection)
            .await?;

        Ok(updated_playlist)
    }

    /// Delete a cloud playlist by ID
    pub async fn delete_cloud_playlist(&mut self, id: &String) -> AnyResult<()> {
        let playlist = CloudPlaylist::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;

        playlist.delete(&mut self.connection).await?;
        Ok(())
    }
}
