use std::path::PathBuf;
use log::info;
use tauri::State;
use ormlite::Model;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    libs::error::AnyResult,
    plugins::db::DBState,
    plugins::cloud::models::*,
    libs::track::Track,
};

/// Result of a cleanup operation
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CleanupResult {
    pub removed_tracks: usize,
    pub removed_cloud_mappings: usize,
    pub removed_cloud_tracks: usize,
}

/// Clean up tracks whose local files no longer exist, including their cloud mappings
#[tauri::command]
pub async fn cleanup_missing_local_tracks(
    db_state: State<'_, DBState>,
) -> AnyResult<CleanupResult> {
    info!("Starting cleanup of tracks with missing local files");
    let mut db = db_state.get_lock().await;
    let mut result = CleanupResult {
        removed_tracks: 0,
        removed_cloud_mappings: 0,
        removed_cloud_tracks: 0,
    };

    // Get all local tracks
    let tracks = Track::select()
        .fetch_all(&mut db.connection)
        .await?;

    for track in tracks {
        let path = PathBuf::from(&track.path);

        // If local file doesn't exist, clean up the track and its related data
        if !path.exists() {
            info!("Local file missing, cleaning up track: {:?}", path);

            // First remove any cloud mappings for this track by matching the path
            let maps = CloudTrackMap::query(r#"
                SELECT ctm.* 
                FROM cloud_track_maps ctm
                INNER JOIN cloud_music_folders cmf ON ctm.cloud_music_folder_id = cmf.id
                WHERE cmf.local_folder_path || '/' || ctm.relative_path = ?"#)
                .bind(&track.path)
                .fetch_all(&mut db.connection)
                .await?;

            for map in maps {
                let cloud_track_id = map.cloud_track_id.clone();
                map.delete(&mut db.connection).await?;
                result.removed_cloud_mappings += 1;

                // Check if cloud track has any remaining mappings
                let remaining_maps = CloudTrackMap::select()
                    .where_("cloud_track_id = ?")
                    .bind(&cloud_track_id)
                    .fetch_optional(&mut db.connection)
                    .await?;

                // If no remaining mappings, remove the cloud track
                if remaining_maps.is_none() {
                    if let Ok(cloud_track) = CloudTrack::select()
                        .where_("id = ?")
                        .bind(&cloud_track_id)
                        .fetch_optional(&mut db.connection)
                        .await
                    {
                        if let Some(cloud_track) = cloud_track {
                            cloud_track.delete(&mut db.connection).await?;
                            result.removed_cloud_tracks += 1;
                            info!("Removed orphaned cloud track: {}", cloud_track_id);
                        }
                    }
                }
            }

            // Finally remove the local track
            track.delete(&mut db.connection).await?;
            result.removed_tracks += 1;
        }
    }

    info!("Cleanup complete. Removed {} tracks, {} cloud mappings, {} orphaned cloud tracks", 
          result.removed_tracks, result.removed_cloud_mappings, result.removed_cloud_tracks);
    Ok(result)
}
