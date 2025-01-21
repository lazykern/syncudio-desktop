use chrono::Utc;
use log::info;
use ormlite::Model;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tauri::State;
use tauri::AppHandle;
use tauri::Error;

use crate::libs::error::SyncudioError;
use crate::libs::utils::blake3_hash;
use crate::plugins::cloud;
use crate::plugins::cloud::CloudProvider;
use crate::plugins::cloud::CloudProviderType;
use crate::plugins::cloud::CloudState;
use crate::plugins::cloud::models::*;
use crate::{libs::error::AnyResult, plugins::db::DBState};

use super::cloud_list_files;

impl SyncStatus {
    pub fn from_str(s: &str) -> AnyResult<Self> {
        match s {
            "pending" => Ok(SyncStatus::Pending),
            "in_progress" => Ok(SyncStatus::InProgress),
            "completed" => Ok(SyncStatus::Completed),
            _ => Ok(SyncStatus::Failed {
                error: "Unknown status".to_string(),
                attempts: 0,
            }),
        }
    }
}

/// Get detailed sync information for a cloud folder
#[tauri::command]
pub async fn get_cloud_folder_sync_details(
    folder_id: String,
    db_state: State<'_, DBState>,
    cloud_state: State<'_, CloudState>,
) -> AnyResult<CloudFolderSyncDetailsDTO> {
    let mut db = db_state.get_lock().await;

    // Get folder info with a single query
    let folder = CloudMusicFolder::select()
        .where_("id = ?")
        .bind(&folder_id)
        .fetch_one(&mut db.connection)
        .await?;

    // Get tracks with their maps in a single query
    let tracks_with_maps: Vec<TrackWithMapRow> = ormlite::query_as(r#"
        SELECT 
            t.id, t.blake3_hash, t.file_name, t.updated_at, t.tags,
            m.id as map_id, m.relative_path, m.cloud_folder_id, m.cloud_file_id
        FROM cloud_tracks t
        INNER JOIN cloud_track_maps m ON t.id = m.cloud_track_id
        WHERE m.cloud_folder_id = ?
    "#)
    .bind(&folder_id)
    .fetch_all(&mut db.connection)
    .await?;

    // Get active operations in a single query
    let active_operations: Vec<QueueOperationRow> = ormlite::query_as(r#"
        SELECT 
            'upload' as queue_type,
            u.status,
            u.cloud_track_map_id
        FROM upload_queue u
        INNER JOIN cloud_track_maps m ON u.cloud_track_map_id = m.id
        WHERE m.cloud_folder_id = ? 
            AND (u.status = 'pending' OR u.status = 'in_progress')
        UNION ALL
        SELECT 
            'download' as queue_type,
            d.status,
            d.cloud_track_map_id
        FROM download_queue d
        INNER JOIN cloud_track_maps m ON d.cloud_track_map_id = m.id
        WHERE m.cloud_folder_id = ?
            AND (d.status = 'pending' OR d.status = 'in_progress')
    "#)
    .bind(&folder_id)
    .bind(&folder_id)
    .fetch_all(&mut db.connection)
    .await?;

    // Create a map for quick operation lookups
    let operation_map: HashMap<String, (&str, &str)> = active_operations
        .iter()
        .map(|op| (
            op.cloud_track_map_id.clone(),
            (op.queue_type.as_str(), op.status.as_str())
        ))
        .collect();

    // Build track DTOs
    let mut track_dtos = Vec::new();
    let mut has_attention_needed = false;

    for track in tracks_with_maps {
        // Check file existence in both locations
        let local_path = Path::new(&folder.local_folder_path)
            .join(&track.relative_path)
            .to_string_lossy()
            .to_string();
        let local_exists = Path::new(&local_path).exists();

        // Calculate location state
        let location_state = match (
            local_exists,
            track.cloud_file_id.is_some(),
            track.blake3_hash.is_some(),
        ) {
            (true, true, true) => TrackLocationState::Complete,
            (true, false, _) => {
                has_attention_needed = true;
                TrackLocationState::LocalOnly
            }
            (false, true, _) => {
                has_attention_needed = true;
                TrackLocationState::CloudOnly
            }
            _ => {
                has_attention_needed = true;
                TrackLocationState::Missing
            }
        };

        // Get sync operation if any
        let (sync_operation, sync_status) = if let Some((op_type, status)) = operation_map.get(&track.map_id) {
            (
                Some(match *op_type {
                    "upload" => SyncOperationType::Upload,
                    "download" => SyncOperationType::Download,
                    _ => unreachable!(),
                }),
                Some(SyncStatus::from_str(status).unwrap()),
            )
        } else {
            (None, None)
        };

        track_dtos.push(CloudTrackDTO {
            id: track.id,
            cloud_folder_id: folder.id.clone(),
            cloud_track_map_id: track.map_id,
            file_name: track.file_name,
            relative_path: track.relative_path,
            location_state,
            sync_operation,
            sync_status,
            updated_at: track.updated_at,
            tags: track.tags,
        });
    }

    // Calculate folder status
    let sync_status = if track_dtos.is_empty() {
        FolderSyncStatus::Empty
    } else if !active_operations.is_empty() {
        FolderSyncStatus::Syncing
    } else if has_attention_needed {
        FolderSyncStatus::NeedsAttention
    } else {
        FolderSyncStatus::Synced
    };

    Ok(CloudFolderSyncDetailsDTO {
        id: folder.id,
        cloud_folder_path: folder.cloud_folder_path,
        local_folder_path: folder.local_folder_path,
        sync_status,
        pending_sync_count: active_operations.len() as i32,
        tracks: track_dtos,
    })
}

/// Command to get sync status for a track
#[tauri::command]
pub async fn get_track_sync_status(
    track_id: String,
    db_state: State<'_, DBState>,
) -> AnyResult<TrackSyncStatusDTO> {
    let mut db = db_state.get_lock().await;

    // Get track and its map
    let track = CloudTrack::select()
        .where_("id = ?")
        .bind(&track_id)
        .fetch_one(&mut db.connection)
        .await?;

    let track_map = CloudTrackMap::select()
        .where_("cloud_track_id = ?")
        .bind(&track_id)
        .fetch_one(&mut db.connection)
        .await?;

    // Get folder to check file existence
    let folder = CloudMusicFolder::select()
        .where_("id = ?")
        .bind(&track_map.cloud_folder_id)
        .fetch_one(&mut db.connection)
        .await?;

    // Check file existence
    let local_path = Path::new(&folder.local_folder_path)
        .join(&track_map.relative_path)
        .to_string_lossy()
        .to_string();
    let local_exists = Path::new(&local_path).exists();

    // Get active operations
    let upload_op = UploadQueueItem::select()
        .where_("cloud_track_map_id = ? AND (status = 'pending' OR status = 'in_progress')")
        .bind(&track_map.id)
        .fetch_optional(&mut db.connection)
        .await?;

    let download_op = DownloadQueueItem::select()
        .where_("cloud_track_map_id = ? AND (status = 'pending' OR status = 'in_progress')")
        .bind(&track_map.id)
        .fetch_optional(&mut db.connection)
        .await?;

    // Calculate location state
    let location_state = match (
        local_exists,
        track_map.cloud_file_id.is_some(),
        track.blake3_hash.is_some(),
    ) {
        (true, true, true) => TrackLocationState::Complete,
        (true, false, true) => TrackLocationState::LocalOnly,
        (false, true, _) => TrackLocationState::CloudOnly,
        (false, false, _) => TrackLocationState::Missing,
        _ => TrackLocationState::NotMapped,
    };

    Ok(TrackSyncStatusDTO {
        location_state,
        sync_operation: upload_op
            .as_ref()
            .map(|_| SyncOperationType::Upload)
            .or_else(|| download_op.as_ref().map(|_| SyncOperationType::Download)),
        sync_status: match (upload_op.as_ref(), download_op.as_ref()) {
            (Some(op), _) => Some(SyncStatus::from_str(&op.status)?),
            (None, Some(op)) => Some(SyncStatus::from_str(&op.status)?),
            (None, None) => None,
        },
        updated_at: track.updated_at,
    })
}