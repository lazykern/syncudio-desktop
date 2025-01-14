use std::path::PathBuf;

use ormlite::model::ModelBuilder;
use ormlite::Model;

use crate::libs::error::AnyResult;
use crate::libs::playlist::Playlist;
use crate::libs::utils::TimeLogger;

use super::core::DB;

impl DB {
    /// Get all the playlists (and their content) from the database
    pub async fn get_all_playlists(&mut self) -> AnyResult<Vec<Playlist>> {
        let timer = TimeLogger::new("Retrieved and decoded playlists".into());
        let mut playlists = Playlist::select()
            .order_asc("name")
            .fetch_all(&mut self.connection)
            .await?;

        // Ensure the playlists are sorted alphabetically (case-insensitive) for better UX
        playlists.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        timer.complete();
        Ok(playlists)
    }

    /// Get a single playlist by ID
    pub async fn get_playlist(&mut self, playlist_id: &String) -> AnyResult<Option<Playlist>> {
        let playlist = Playlist::select()
            .where_bind("id = ?", playlist_id)
            .fetch_one(&mut self.connection)
            .await?;

        Ok(Some(playlist))
    }

    /// Create a playlist given a name and a set of track IDs
    pub async fn create_playlist(
        &mut self,
        name: String,
        tracks_ids: Vec<String>,
        import_path: Option<PathBuf>,
    ) -> AnyResult<Playlist> {
        let playlist_path: Option<String> =
            import_path.map(|path| path.to_str().unwrap().to_string());

        let playlist = Playlist {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            tracks: tracks_ids,
            import_path: playlist_path,
        };

        let playlist = playlist.insert(&mut self.connection).await?;

        Ok(playlist)
    }

    /// Set the tracks of a playlist given its ID and tracks IDs
    pub async fn set_playlist_tracks(
        &mut self,
        id: &String,
        tracks: Vec<String>,
    ) -> AnyResult<Playlist> {
        let playlist = Playlist::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;

        let updated_playlist = playlist
            .update_partial()
            .tracks(tracks)
            .update(&mut self.connection)
            .await?;

        Ok(updated_playlist)
    }

    /// Update a playlist name by ID
    pub async fn rename_playlist(&mut self, id: &String, name: String) -> AnyResult<Playlist> {
        let playlist = Playlist::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;

        let updated_playlist = playlist
            .update_partial()
            .name(name)
            .update(&mut self.connection)
            .await?;

        Ok(updated_playlist)
    }

    /// Delete a playlist by ID
    pub async fn delete_playlist(&mut self, id: &String) -> AnyResult<()> {
        let playlist = Playlist::select()
            .where_bind("id = ?", id)
            .fetch_one(&mut self.connection)
            .await?;

        playlist.delete(&mut self.connection).await?;

        Ok(())
    }
} 