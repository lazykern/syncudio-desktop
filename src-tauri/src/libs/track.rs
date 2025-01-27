use std::path::PathBuf;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, ItemKey};
use log::warn;
use ormlite::model::Model;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::utils::blake3_hash;

/**
 * Track
 * represent a single track, id and path should be unique
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "tracks")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct Track {
    #[ormlite(primary_key)]
    pub id: String,
    pub blake3_hash: Option<String>,
    pub path: String, // must be unique, ideally, a PathBuf
    pub title: String,
    pub album: String,
    #[ormlite(json)]
    pub artists: Vec<String>,
    #[ormlite(json)]
    pub composers: Vec<String>,
    #[ormlite(json)]
    pub album_artists: Vec<String>,
    #[ormlite(json)]
    pub genres: Vec<String>,
    pub track_no: Option<u32>,
    pub track_of: Option<u32>,
    pub disk_no: Option<u32>,
    pub disk_of: Option<u32>,
    pub date: Option<String>,
    pub year: Option<u32>,
    pub duration: u32,
    pub bitrate: Option<u32>,
    pub sampling_rate: Option<u32>,
    pub channels: Option<u32>,
    pub encoder: Option<String>,
}

/**
 * Generate a Track struct from a Path, or nothing if it is not a valid audio
 * file
 */
pub fn get_track_from_file(path: &PathBuf) -> Option<Track> {
    match lofty::read_from_path(path) {
        Ok(tagged_file) => {
            let tag = tagged_file.primary_tag()?;
            let properties = tagged_file.properties();
            let metadata = std::fs::metadata(path).ok()?;

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

            let composers: Vec<String> = tag
                .get_strings(&ItemKey::Composer)
                .map(ToString::to_string)
                .collect();

            let album_artists: Vec<String> = tag
                .get_strings(&ItemKey::AlbumArtist)
                .map(ToString::to_string)
                .collect();

            let id = get_track_id_for_path(path)?;

            Some(Track {
                id,
                blake3_hash: blake3_hash(path).ok(),
                path: path.to_string_lossy().into_owned(),
                title: tag
                    .get_string(&ItemKey::TrackTitle)
                    .unwrap_or("Unknown")
                    .to_string(),
                album: tag
                    .get_string(&ItemKey::AlbumTitle)
                    .unwrap_or("Unknown")
                    .to_string(),
                artists,
                composers,
                album_artists,
                genres: tag
                    .get_strings(&ItemKey::Genre)
                    .map(ToString::to_string)
                    .collect(),
                track_no: tag.track(),
                track_of: tag.track_total(),
                disk_no: tag.disk(),
                disk_of: tag.disk_total(),
                date: tag.get_string(&ItemKey::ReleaseDate).map(String::from),
                year: tag.year(),
                duration: u32::try_from(properties.duration().as_secs()).unwrap_or(0),
                bitrate: properties.audio_bitrate(),
                sampling_rate: properties.sample_rate(),
                channels: properties.channels().map(|c| c as u32),
                encoder: tag.get_string(&ItemKey::EncodedBy).map(String::from),
            })
        }
        Err(err) => {
            warn!("Failed to get ID3 tags: \"{}\". File {:?}", err, path);
            None
        }
    }
}

/**
 * Generate an ID for a track based on its location.
 *
 * We leverage UUID v3 on tracks paths to easily retrieve tracks by path.
 * This is not great and ideally we should use a DB view instead. One day.
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
