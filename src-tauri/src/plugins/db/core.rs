use log::{error, info};
use ormlite::sqlite::{SqliteConnectOptions, SqliteConnection};
use ormlite::{Connection, TableMeta};
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime, State};
use tokio::sync::{Mutex, MutexGuard};

use crate::libs::database::DB;
use crate::libs::error::AnyResult;
use crate::libs::playlist::Playlist;
use crate::libs::track::Track;
use crate::libs::utils::TimeLogger;
use crate::plugins::config::get_storage_dir;

use super::track::*;
use super::playlist::*;
use super::library::*;
/// Database state wrapper to ensure thread safety
pub struct DBState(Mutex<DB>);

impl DBState {
    /// Get a lock on the database connection
    pub async fn get_lock(&self) -> MutexGuard<'_, DB> {
        self.0.lock().await
    }
}

/// Setup the database connection and create tables
async fn setup() -> AnyResult<DB> {
    let database_path = get_storage_dir().join("syncudio.db");

    info!("Opening connection to database: {:?}", database_path);

    let options = SqliteConnectOptions::new()
        .filename(&database_path)
        .create_if_missing(true)
        .optimize_on_close(true, None)
        .auto_vacuum(ormlite::sqlite::SqliteAutoVacuum::Incremental)
        .journal_mode(ormlite::sqlite::SqliteJournalMode::Wal);

    let connection = SqliteConnection::connect_with(&options).await?;

    Ok(DB { connection })
}

/// Reset the database by deleting all data
#[tauri::command]
pub async fn reset(db_state: State<'_, DBState>) -> AnyResult<()> {
    info!("Resetting DB...");
    let timer = TimeLogger::new("Reset DB".into());

    let mut db = db_state.get_lock().await;

    let delete_tracks_query = format!("DELETE FROM {};", Track::table_name());
    let delete_playlists_query = format!("DELETE FROM {};", Playlist::table_name());

    ormlite::query(&delete_tracks_query)
        .execute(&mut db.connection)
        .await?;
    ormlite::query(&delete_playlists_query)
        .execute(&mut db.connection)
        .await?;
    ormlite::query("VACUUM;")
        .execute(&mut db.connection)
        .await?;

    timer.complete();

    Ok(())
}

/// Initialize the database plugin
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("database")
        .invoke_handler(tauri::generate_handler![
            // Track operations
            get_all_tracks,
            get_tracks,
            remove_tracks,
            update_track,
            import_tracks_to_library,
            // Playlist operations
            get_all_playlists,
            get_playlist,
            create_playlist,
            rename_playlist,
            set_playlist_tracks,
            export_playlist,
            delete_playlist,
            // Core operations
            reset,
        ])
        .setup(move |app_handle, _api| {
            let app_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let mut db = match setup().await {
                    Ok(db) => db,
                    Err(err) => {
                        error!("Failed to setup database: {:?}", err);
                        return;
                    }
                };

                db.create_tables()
                    .await
                    .expect("Could not create DB tables");

                app_handle.manage(DBState(Mutex::new(db)));
            });
            Ok(())
        })
        .build()
} 