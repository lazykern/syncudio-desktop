use anyhow::Result;
use lofty::error::LoftyError;
use serde::{ser::Serializer, Serialize};
use thiserror::Error;

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
    Unknown(#[from] anyhow::Error),

    #[error("Dropbox SDK error: {0}")]
    DropboxSdk(String),

    #[error("Dropbox error: {0}")]
    Dropbox(String),

    #[error("Google Drive error: {0}")]
    GoogleDrive(String),

    /**
     * Custom errors
     */
    #[error("Playlist not found")]
    PlaylistNotFound,

    #[error("Invalid provider type")]
    InvalidProviderType,
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
