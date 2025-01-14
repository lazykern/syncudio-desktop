use std::collections::HashMap;

use ormlite::model::ModelBuilder;
use ormlite::Model;

use crate::libs::error::AnyResult;
use crate::libs::track::Track;
use crate::libs::utils::TimeLogger;

use super::core::DB;

impl DB {
    /// Get all the tracks (and their content) from the database
    pub async fn get_all_tracks(&mut self) -> AnyResult<Vec<Track>> {
        let timer = TimeLogger::new("Retrieved and decoded tracks".into());
        let tracks = Track::select().fetch_all(&mut self.connection).await?;
        timer.complete();
        Ok(tracks)
    }

    /// Get tracks (and their content) given a set of document IDs
    pub async fn get_tracks(&mut self, track_ids: &Vec<String>) -> AnyResult<Vec<Track>> {
        // TODO: Can this be improved somehow?
        // Improve me once https://github.com/launchbadge/sqlx/issues/875 is fixed
        let placeholders = track_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let where_statement = format!("id IN ({})", placeholders);

        let mut query_builder = Track::select().dangerous_where(&where_statement);

        for id in track_ids {
            query_builder = query_builder.bind(id);
        }

        let mut tracks: Vec<Track> = query_builder.fetch_all(&mut self.connection).await?;

        // document may not ordered the way we want, so let's ensure they map to track_ids
        let track_id_positions: HashMap<&String, usize> = track_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id, i))
            .collect();
        tracks.sort_by_key(|track| track_id_positions.get(&track.id));

        Ok(tracks)
    }

    /// Update a track in the database
    pub async fn update_track(&mut self, track: Track) -> AnyResult<Track> {
        let updated_track = track.update_all_fields(&mut self.connection).await?;
        Ok(updated_track)
    }

    /// Delete multiple tracks by ID
    pub async fn remove_tracks(&mut self, track_ids: &Vec<String>) -> AnyResult<()> {
        // TODO: batch that, use DELETE statement instead
        let tracks = self.get_tracks(track_ids).await?;

        for track in tracks {
            track.delete(&mut self.connection).await?
        }

        Ok(())
    }

    /// Insert a new track in the DB, will fail in case there is a duplicate unique
    /// key (like track.path)
    ///
    /// Doc: https://github.com/khonsulabs/bonsaidb/blob/main/examples/basic-local/examples/basic-local-multidb.rs
    pub async fn insert_tracks(&mut self, tracks: Vec<Track>) -> AnyResult<()> {
        // Weirdly, this is fast enough with SQLite, no need to create transactions
        for track in tracks {
            track.insert(&mut self.connection).await?;
        }

        Ok(())
    }
} 