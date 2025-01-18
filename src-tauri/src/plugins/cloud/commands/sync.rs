use uuid::Uuid;
use chrono::Utc;
use log::info;
use ormlite::Model;
use std::collections::HashMap;
use std::path::Path;
use tauri::State;

use crate::{
    libs::{
        error::{AnyResult, SyncudioError},
        utils::normalize_relative_path,
    },
    plugins::{
        cloud::{
            models::{
                cloud_folder::CloudFolder, cloud_track::{CloudTrack, CloudTrackMap}, dto::{
                    CloudFolderSyncDetailsDTO, CloudTrackDTO, FolderSyncStatus, QueueItemDTO,
                    QueueStatsDTO, SyncOperationType, SyncStatus, TrackLocationState,
                    TrackSyncStatusDTO,
                }, query_models::{QueueOperationRow, QueueStatsRow, TrackWithMapRow}, sync_queue::{DownloadQueueItem, UploadQueueItem}
            },
            providers::CloudFile,
            CloudState,
        },
        db::DBState,
    },
};

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
    let folder = db.get_cloud_folder(&folder_id).await?.ok_or(SyncudioError::NotFound("Cloud folder".into()))?;

    // Get tracks with their maps in a single query
    let tracks_with_maps: Vec<TrackWithMapRow> = ormlite::query_as(r#"
        SELECT 
            t.id, t.blake3_hash, t.cloud_file_id, t.file_name, t.updated_at, t.tags,
            m.id as map_id, m.relative_path, m.cloud_folder_id
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

/// Get active queue items
#[tauri::command]
pub async fn get_queue_items(
    folder_id: Option<String>,
    db_state: State<'_, DBState>,
) -> AnyResult<Vec<QueueItemDTO>> {
    let mut db = db_state.get_lock().await;
    let mut items = Vec::new();

    // Get upload queue items
    let upload_items = if let Some(folder_id) = &folder_id {
        db.get_upload_queue_items_by_folder_id(&folder_id).await?
    } else {
        db.get_upload_queue_items().await?
    };

    // Get download queue items
    let download_items = if let Some(folder_id) = &folder_id {
        db.get_download_queue_items_by_folder_id(&folder_id).await?
    } else {
        db.get_download_queue_items().await?
    };

    // Convert upload items to DTOs
    for item in upload_items {
        // Get track info
        let track_map = db.get_cloud_track_map(&item.cloud_track_map_id).await?;
        let track = if let Some(map) = track_map {
            db.get_cloud_track(&map.cloud_track_id).await?
        } else {
            None
        };

        if let Some(track) = track {
            items.push(QueueItemDTO {
                id: item.id,
                cloud_track_id: track.id,
                file_name: track.file_name,
                operation: SyncOperationType::Upload,
                status: if item.status == "failed" {
                    SyncStatus::Failed {
                        error: item.error_message.unwrap_or_else(|| "Unknown error".to_string()),
                        attempts: item.attempts,
                    }
                } else {
                    SyncStatus::from_str(&item.status)?
                },
                created_at: item.created_at,
                updated_at: item.updated_at,
                provider_type: item.provider_type,
            });
        }
    }

    // Convert download items to DTOs
    for item in download_items {
        // Get track info
        let track_map = db.get_cloud_track_map(&item.cloud_track_map_id).await?;
        let track = if let Some(map) = track_map {
            db.get_cloud_track(&map.cloud_track_id).await?
        } else {
            None
        };

        if let Some(track) = track {
            items.push(QueueItemDTO {
                id: item.id,
                cloud_track_id: track.id,
                file_name: track.file_name,
                operation: SyncOperationType::Download,
                status: if item.status == "failed" {
                    SyncStatus::Failed {
                        error: item.error_message.unwrap_or_else(|| "Unknown error".to_string()),
                        attempts: item.attempts,
                    }
                } else {
                    SyncStatus::from_str(&item.status)?
                },
                created_at: item.created_at,
                updated_at: item.updated_at,
                provider_type: item.provider_type,
            });
        }
    }

    // Sort all items by created_at descending
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(items)
}

/// Get queue statistics
#[tauri::command]
pub async fn get_queue_stats(
    folder_id: Option<String>,
    db_state: State<'_, DBState>,
) -> AnyResult<QueueStatsDTO> {
    let mut db = db_state.get_lock().await;
    let folder_id_ref = folder_id.as_deref();
    
    let stats = db.get_queue_stats(folder_id_ref).await?;
    
    let mut dto = QueueStatsDTO {
        pending_count: 0,
        in_progress_count: 0,
        completed_count: 0,
        failed_count: 0,
    };

    for stat in stats {
        match stat.status.as_str() {
            "pending" => dto.pending_count = stat.count,
            "in_progress" => dto.in_progress_count = stat.count,
            "completed" => dto.completed_count = stat.count,
            "failed" => dto.failed_count = stat.count,
            _ => {}
        }
    }

    Ok(dto)
}

/// Command to pause/resume sync operations
#[tauri::command]
pub async fn set_sync_paused(paused: bool, cloud_state: State<'_, CloudState>) -> AnyResult<()> {
    todo!("Implement set_sync_paused");
    // 1. Update sync worker state
    // 2. Pause/resume active transfers
}

/// Command to retry failed sync items
#[tauri::command]
pub async fn retry_failed_items(
    folder_id: Option<String>,
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    todo!("Implement retry_failed_items");
    // 1. Get failed items for folder (or all if none specified)
    // 2. Reset status and attempt count
    // 3. Requeue items
}

/// Command to get sync status for a track
#[tauri::command]
pub async fn get_track_sync_status(
    track_id: String,
    db_state: State<'_, DBState>,
) -> AnyResult<TrackSyncStatusDTO> {
    let mut db = db_state.get_lock().await;

    // Get track and its map
    let track = db.get_cloud_track(&track_id).await?.ok_or(SyncudioError::NotFound("Cloud track".into()))?;

    let track_map = db.get_cloud_track_map_by_track_id(&track_id).await?.ok_or(SyncudioError::NotFound("Cloud track map".into()))?;

    // Get folder to check file existence
    let folder = db.get_cloud_folder(&track_map.cloud_folder_id).await?.ok_or(SyncudioError::NotFound("Cloud folder".into()))?;

    // Check file existence
    let local_path = Path::new(&folder.local_folder_path)
        .join(&track_map.relative_path)
        .to_string_lossy()
        .to_string();
    let local_exists = Path::new(&local_path).exists();

    // Get active operations
    let upload_op = db.get_active_upload_queue_item(&track_map.id).await?;
    let download_op = db.get_active_download_queue_item(&track_map.id).await?;

    // Calculate location state
    let location_state = match (
        local_exists,
        track.cloud_file_id.is_some(),
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

/// Add tracks to the upload queue
#[tauri::command]
pub async fn add_to_upload_queue(
    track_ids: Vec<String>,
    folder_id: String,
    priority: Option<i32>,
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    let mut db = db_state.get_lock().await;
    let now = Utc::now();

    for track_id in track_ids {
        // Get track and its map
        let track_map = db.get_cloud_track_map_by_track_and_folder(&track_id, &folder_id).await?.ok_or(SyncudioError::NotFound("Cloud track map".into()))?;

        // Check for existing active operations
        let has_active_upload = db.get_active_upload_queue_item(&track_map.id).await?.is_some();
        let has_active_download = db.get_active_download_queue_item(&track_map.id).await?.is_some();

        if has_active_upload || has_active_download {
            // Skip this track as it already has an active operation
            info!("Skipping track {} as it already has an active sync operation", track_id);
            continue;
        }

        // Get folder to determine provider type
        let folder = db.get_cloud_folder(&track_map.cloud_folder_id).await?.ok_or(SyncudioError::NotFound("Cloud folder".into()))?;

        // Create upload queue item
        let upload_item = UploadQueueItem {
            id: Uuid::new_v4().to_string(),
            priority: priority.unwrap_or(0),
            cloud_track_map_id: track_map.id,
            provider_type: folder.provider_type,
            status: "pending".to_string(),
            error_message: None,
            created_at: now,
            updated_at: now,
            attempts: 0,
        };

        upload_item.insert(&mut db.connection).await?;
    }

    Ok(())
}

/// Add tracks to the download queue
#[tauri::command]
pub async fn add_to_download_queue(
    track_ids: Vec<String>,
    folder_id: String,
    priority: Option<i32>,
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    let mut db = db_state.get_lock().await;
    let now = Utc::now();

    for track_id in track_ids {
        // Get track and its map
        let track = db.get_cloud_track(&track_id).await?.ok_or(SyncudioError::NotFound("Cloud track".into()))?;

        let track_map = db.get_cloud_track_map_by_track_and_folder(&track_id, &folder_id).await?.ok_or(SyncudioError::NotFound("Cloud track map".into()))?;

        // Check for existing active operations
        let has_active_upload = db.get_active_upload_queue_item(&track_map.id).await?.is_some();
        let has_active_download = db.get_active_download_queue_item(&track_map.id).await?.is_some();

        if has_active_upload || has_active_download {
            continue;
        }

        // Get folder to determine provider type
        let folder = db.get_cloud_folder(&track_map.cloud_folder_id).await?.ok_or(SyncudioError::NotFound("Cloud folder".into()))?;

        // Add to download queue
        let download_item = DownloadQueueItem {
            id: Uuid::new_v4().to_string(),
            cloud_track_map_id: track_map.id.clone(),
            provider_type: folder.provider_type,
            priority: priority.unwrap_or(0),
            status: "pending".to_string(),
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attempts: 0,
        };

        download_item.insert(&mut db.connection).await?;
    }

    Ok(())
}