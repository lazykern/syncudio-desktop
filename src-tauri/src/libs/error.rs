use anyhow::Result;
use lofty::error::LoftyError;
use serde::{ser::Serializer, Serialize};
use thiserror::Error;
use std::path::StripPrefixError;

/**
 * Create the error type that represents all errors possible in our program
 * Stolen from https://github.com/tauri-apps/tauri/discussions/3913
 */
#[derive(Debug, Error)]
pub enum SyncudioError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Tauri(#[from] tauri::Error),

    #[error(transparent)]
    Lofty(#[from] LoftyError),

    #[error(transparent)]
    ORMLite(#[from] ormlite::Error),

    #[error(transparent)]
    ORMLiteSqlx(#[from] ormlite::SqlxError),

    #[error(transparent)]
    NoSleep(#[from] nosleep::Error),

    #[error("An error occurred while manipulating the config: {0}")]
    Config(String),

    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),

    #[error("Dropbox SDK error: {0}")]
    DropboxSdk(String),

    #[error("Dropbox error: {0}")]
    Dropbox(String),

    #[error("Google Drive error: {0}")]
    GoogleDrive(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /**
     * Custom errors
     */
    #[error("Playlist not found")]
    PlaylistNotFound,

    #[error("Invalid provider type")]
    InvalidProviderType,

    #[error("Path error: {0}")]
    Path(String),

    #[error("Invalid queue status")]
    InvalidQueueStatus,

    #[error("Last.fm error: {0}")]
    LastFm(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid track metadata: {0}")]
    InvalidTrackMetadata(String),

    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),
}

/**
 * Let's make anyhow's errors Tauri friendly, so they can be used for commands
 */
impl Serialize for SyncudioError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type AnyResult<T, E = SyncudioError> = Result<T, E>;

impl<T: std::fmt::Debug> From<dropbox_sdk::Error<T>> for SyncudioError {
    fn from(error: dropbox_sdk::Error<T>) -> Self {
        SyncudioError::DropboxSdk(format!("{:?}", error))
    }
}

impl From<StripPrefixError> for SyncudioError {
    fn from(error: StripPrefixError) -> Self {
        SyncudioError::Path(error.to_string())
    }
}

impl From<serde_json::Error> for SyncudioError {
    fn from(error: serde_json::Error) -> Self {
        SyncudioError::SerializationError(error.to_string())
    }
}

impl From<toml::ser::Error> for SyncudioError {
    fn from(error: toml::ser::Error) -> Self {
        SyncudioError::SerializationError(error.to_string())
    }
}
