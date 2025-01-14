use std::path::PathBuf;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, ItemKey};
use log::warn;
use ormlite::model::Model;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/**
 * Track
 * represent a single track
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "tracks")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct Track {
    #[ormlite(primary_key)]
    pub id: String,
    pub local_folder_path: String,  // References local_folders.path
    pub relative_path: String,      // Path relative to local_folder_path
    pub title: String,
    pub album: String,
    #[ormlite(json)]
    pub artists: Vec<String>,
    #[ormlite(json)]
    pub genres: Vec<String>,
    pub year: Option<u32>,
    pub duration: u32,
    pub track_no: Option<u32>,
    pub track_of: Option<u32>,
    pub disk_no: Option<u32>,
    pub disk_of: Option<u32>,
    pub index_hash: Option<String>,
}

impl Track {
    pub fn path(&self) -> PathBuf {
        PathBuf::from(&self.local_folder_path).join(&self.relative_path)
    }
    
    pub fn with_index_hash(mut self, hash: String) -> Self {
        self.index_hash = Some(hash);
        self
    }
}

/**
 * Generate a Track struct from a Path and its local folder, or nothing if it is not a valid audio file
 */
pub fn get_track_from_file(abs_path: &PathBuf, local_folder_path: &str) -> Option<Track> {
    match lofty::read_from_path(abs_path) {
        Ok(tagged_file) => {
            let tag = tagged_file.primary_tag()?;

            let mut artists: Vec<String> = tag
                .get_strings(&ItemKey::TrackArtist)
                .map(ToString::to_string)
                .collect();

            if artists.is_empty() {
                artists = tag
                    .get_strings(&ItemKey::AlbumArtist)
                    .map(ToString::to_string)
                    .collect();
            }

            if artists.is_empty() {
                artists = vec!["Unknown Artist".into()];
            }

            let id = get_track_id_for_path(abs_path)?;
            
            // Convert absolute path to relative path
            let abs_folder = PathBuf::from(local_folder_path);
            let relative_path = abs_path.strip_prefix(&abs_folder).ok()?.to_string_lossy().into_owned();

            Some(Track {
                id,
                local_folder_path: local_folder_path.to_string(),
                relative_path,
                title: tag
                    .get_string(&ItemKey::TrackTitle)
                    .unwrap_or("Unknown")
                    .to_string(),
                album: tag
                    .get_string(&ItemKey::AlbumTitle)
                    .unwrap_or("Unknown")
                    .to_string(),
                artists,
                genres: tag
                    .get_strings(&ItemKey::Genre)
                    .map(ToString::to_string)
                    .collect(),
                year: tag.year(),
                duration: u32::try_from(tagged_file.properties().duration().as_secs()).unwrap_or(0),
                track_no: tag.track(),
                track_of: tag.track_total(),
                disk_no: tag.disk(),
                disk_of: tag.disk_total(),
                index_hash: None,
            })
        }
        Err(err) => {
            warn!("Failed to get ID3 tags: \"{}\". File {:?}", err, abs_path);
            None
        }
    }
}

/**
 * Generate an ID for a track based on its location.
 * We leverage UUID v3 on tracks paths to easily retrieve tracks by path.
 */
pub fn get_track_id_for_path(path: &PathBuf) -> Option<String> {
    match std::fs::canonicalize(path) {
        Ok(canonicalized_path) => Some(
            Uuid::new_v3(
                &Uuid::NAMESPACE_OID,
                canonicalized_path.to_string_lossy().as_bytes(),
            )
            .to_string(),
        ),
        Err(err) => {
            warn!(r#"ID could not be generated for path {:?}: {}"#, path, err);
            None
        }
    }
}
