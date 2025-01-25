use std::path::PathBuf;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime, State};
use log::info;
use rustfm_scrobble::{Scrobble, Scrobbler};

use crate::libs::error::{AnyResult, SyncudioError};
use crate::plugins::config::get_storage_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastFmSession {
    pub username: String,
    pub session_key: String,
}

pub struct LastFmManager {
    client: RwLock<Scrobbler>,
    storage_path: PathBuf,
}

impl LastFmManager {
    pub fn new() -> Self {
        let storage_dir = get_storage_dir();
        let session_path = storage_dir.join("lastfm_auth.dat");

        let scrobbler = Scrobbler::new(
            "99ae73a3fcccd31a29d59ec6ae09973e",
            "52a7e55d422bc284b125c1d3acfcd836",
        );

        let manager = Self {
            client: RwLock::new(scrobbler),
            storage_path: session_path,
        };

        if let Ok(session_str) = std::fs::read_to_string(&manager.storage_path) {
            if let Ok(session) = toml::from_str::<LastFmSession>(&session_str) {
                info!("Found Last.fm session for user: {}", session.username);
                if let Ok(mut client) = manager.client.write() {
                    client.authenticate_with_session_key(&session.session_key)
                }
            }
        }

        manager
    }

    fn save_session(&self, username: &str, session_key: &str) -> AnyResult<()> {
        let session = LastFmSession {
            username: username.to_string(),
            session_key: session_key.to_string(),
        };
        
        let session_str = toml::to_string(&session)?;
        std::fs::write(&self.storage_path, session_str)?;
        info!("Saved Last.fm session for user: {}", username);
        Ok(())
    }

    pub fn clear_session(&self) -> AnyResult<()> {
        if self.storage_path.exists() {
            std::fs::remove_file(&self.storage_path)?;
            info!("Cleared Last.fm session");
        }
        Ok(())
    }

    pub fn get_session(&self) -> Option<LastFmSession> {
        if let Ok(session_str) = std::fs::read_to_string(&self.storage_path) {
            if let Ok(session) = toml::from_str::<LastFmSession>(&session_str) {
                if !session.session_key.is_empty() {
                    return Some(session);
                }
            }
        }
        None
    }
}

#[tauri::command]
async fn authenticate(
    manager: State<'_, LastFmManager>,
    username: String,
    password: String,
) -> AnyResult<String> {
    let mut client = manager.client.write().map_err(|e| SyncudioError::LastFm(e.to_string()))?;
    
    let session = match client.authenticate_with_password(&username, &password) {
        Ok(auth_response) => {
            manager.save_session(&username, &auth_response.key)?;
            info!("Authenticated with Last.fm for user: {}", username);
            username
        },
        Err(e) => {
            return Err(SyncudioError::LastFm(e.to_string()).into());
        }
    };

    Ok(session)
}

#[tauri::command]
async fn logout(manager: State<'_, LastFmManager>) -> AnyResult<()> {
    manager.clear_session()?;
    Ok(())
}

#[tauri::command]
async fn get_session(manager: State<'_, LastFmManager>) -> AnyResult<Option<LastFmSession>> {
    Ok(manager.get_session())
}

#[tauri::command]
async fn scrobble_track(
    manager: State<'_, LastFmManager>,
    artist: String,
    title: String,
    album: Option<String>,
) -> AnyResult<()> {
    let client = manager.client.read().map_err(|e| SyncudioError::LastFm(e.to_string()))?;

    let scrobble = match album {
        Some(album_name) => {
            let album_str: &str = &album_name;
            Scrobble::new(&artist, &title, album_str)
        },
        None => Scrobble::new(&artist, &title, ""),
    };

    client.scrobble(&scrobble).map_err(|e| SyncudioError::LastFm(e.to_string()))?;
    info!("Scrobbled track: {} - {}", title, artist);
    Ok(())
}

#[tauri::command]
async fn update_now_playing(
    manager: State<'_, LastFmManager>,
    artist: String,
    title: String,
    album: Option<String>,
) -> AnyResult<()> {
    let client = manager.client.read().map_err(|e| SyncudioError::LastFm(e.to_string()))?;

    let scrobble = match album {
        Some(album_name) => {
            let album_str: &str = &album_name;
            Scrobble::new(&artist, &title, album_str)
        },
        None => Scrobble::new(&artist, &title, ""),
    };

    client.now_playing(&scrobble).map_err(|e| SyncudioError::LastFm(e.to_string()))?;
    info!("Updated now playing: {} - {}", title, artist);
    Ok(())
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("lastfm")
        .invoke_handler(tauri::generate_handler![
            authenticate,
            logout,
            get_session,
            scrobble_track,
            update_now_playing,
        ])
        .setup(|app_handle, _api| {
            app_handle.manage(LastFmManager::new());
            Ok(())
        })
        .build()
}
