use uuid::Uuid;
use chrono::Utc;
use log::info;
use ormlite::Model;
use std::collections::HashMap;
use std::path::Path;
use tauri::State;

use crate::{
    libs::{
        error::AnyResult,
        utils::normalize_relative_path,
    },
    plugins::{
        cloud::{
            models::{
                cloud_folder::CloudFolder,
                cloud_track::{CloudTrack, CloudTrackMap},
                dto::{
                    CloudFolderSyncDetailsDTO, CloudTrackDTO, FolderSyncStatus, QueueItemDTO,
                    QueueStatsDTO, SyncOperationType, SyncStatus, TrackLocationState,
                    TrackSyncStatusDTO,
                },
                sync_queue::{DownloadQueueItem, UploadQueueItem},
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

    // 1. Get folder info from database
    let folder = CloudFolder::select()
        .where_("id = ?")
        .bind(&folder_id)
        .fetch_one(&mut db.connection)
        .await?;


    let tracks = CloudTrack::query(r#"SELECT * FROM cloud_tracks WHERE cloud_tracks.id IN (
        SELECT cloud_track_maps.cloud_track_id FROM cloud_track_maps WHERE cloud_track_maps.cloud_folder_id = ?
    )"#)
        .bind(&folder_id)
        .fetch_all(&mut db.connection)
        .await?;

    // 3. Get all cloud files recursively
    let cloud_files = cloud_list_files(
        folder.provider_type.clone(),
        folder.cloud_folder_id.clone(),
        folder.cloud_folder_path.clone(),
        true,
        cloud_state,
    )
    .await?;

    // Create a map of relative paths to cloud files
    let cloud_files_map: HashMap<String, CloudFile> = cloud_files
        .into_iter()
        .filter(|file| !file.is_folder)
        .map(|file| (normalize_relative_path(&file.relative_path), file))
        .collect();

    // 4. Get active sync operations
    let upload_operations = UploadQueueItem::query(
        r#"SELECT * FROM upload_queue WHERE upload_queue.cloud_track_map_id IN (
        SELECT cloud_track_maps.id FROM cloud_track_maps WHERE cloud_track_maps.cloud_folder_id = ?
    ) AND (upload_queue.status = 'pending' OR upload_queue.status = 'in_progress')"#,
    )
    .bind(&folder_id)
    .fetch_all(&mut db.connection)
    .await?;

    let download_operations = DownloadQueueItem::query(
        r#"SELECT * FROM download_queue WHERE download_queue.cloud_track_map_id IN (
        SELECT cloud_track_maps.id FROM cloud_track_maps WHERE cloud_track_maps.cloud_folder_id = ?
    ) AND (download_queue.status = 'pending' OR download_queue.status = 'in_progress')"#,
    )
    .bind(&folder_id)
    .fetch_all(&mut db.connection)
    .await?;

    // 5. Calculate track states and build DTOs
    let mut track_dtos = Vec::new();
    let mut has_attention_needed = false;
    let pending_sync_count = upload_operations.len() + download_operations.len();
    let is_empty = tracks.is_empty();

    for track in tracks {
        let map = CloudTrackMap::select()
            .where_("cloud_track_id = ? AND cloud_folder_id = ?")
            .bind(&track.id)
            .bind(&folder_id)
            .fetch_optional(&mut db.connection)
            .await?;

        if map.is_none() {
            // TODO: Handle this case
            info!("Track map not found: {:?}", track.id);
            continue;
        }

        let map = map.unwrap();

        // Check file existence in both locations
        let local_path = Path::new(&folder.local_folder_path)
            .join(normalize_relative_path(map.relative_path.as_str()))
            .to_string_lossy()
            .to_string();
        let local_exists = Path::new(&local_path).exists();
        let cloud_exists = cloud_files_map.contains_key(&normalize_relative_path(map.relative_path.as_str()));

        // Find any active operations
        let upload_op = upload_operations
            .iter()
            .find(|op| op.cloud_track_map_id == map.id);
        let download_op = download_operations
            .iter()
            .find(|op| op.cloud_track_map_id == map.id);

        // Calculate location state
        let location_state = match (
            local_exists,
            cloud_exists,
            track.blake3_hash.as_ref(),
            track.cloud_file_id.as_ref(),
        ) {
            (true, true, Some(_), Some(_)) => TrackLocationState::Complete,
            (true, false, Some(_), _) => {
                info!("Track local only: {:?} {:?} {:?}", folder.local_folder_path, folder.cloud_folder_path, local_path);
                has_attention_needed = true;
                TrackLocationState::LocalOnly
            }
            (false, true, _, Some(_)) => {
                has_attention_needed = true;
                TrackLocationState::CloudOnly
            }
            (false, false, _, _) => {
                info!("Track missing: {:?} {:?}", folder.local_folder_path, local_path);
                info!("Track map: {:?}", map);
                has_attention_needed = true;
                TrackLocationState::Missing
            }
            _ => {
                has_attention_needed = true;
                TrackLocationState::NotMapped
            }
        };

        // Build track DTO
        track_dtos.push(CloudTrackDTO {
            id: track.id,
            cloud_folder_id: folder.id.clone(),
            cloud_track_map_id: map.id,
            file_name: track.file_name,
            relative_path: map.relative_path.clone(),
            location_state,
            sync_operation: upload_op
                .map(|_| SyncOperationType::Upload)
                .or_else(|| download_op.map(|_| SyncOperationType::Download)),
            sync_status: match (upload_op, download_op) {
                (Some(op), _) => Some(SyncStatus::from_str(&op.status).unwrap()),
                (None, Some(op)) => Some(SyncStatus::from_str(&op.status).unwrap()),
                (None, None) => None,
            },
            updated_at: track.updated_at,
            tags: track.tags,
        });
    }

    // 6. Calculate folder status
    let sync_status = if is_empty {
        FolderSyncStatus::Empty
    } else if pending_sync_count > 0 {
        FolderSyncStatus::Syncing
    } else if has_attention_needed {
        FolderSyncStatus::NeedsAttention
    } else {
        FolderSyncStatus::Synced
    };

    // 7. Return final DTO
    Ok(CloudFolderSyncDetailsDTO {
        id: folder.id,
        cloud_folder_path: folder.cloud_folder_path,
        local_folder_path: folder.local_folder_path,
        sync_status,
        pending_sync_count: pending_sync_count as i32,
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
    let upload_query = if let Some(folder_id) = &folder_id {
        UploadQueueItem::query(
            r#"SELECT * FROM upload_queue WHERE upload_queue.cloud_track_map_id IN (
                SELECT cloud_track_maps.id FROM cloud_track_maps WHERE cloud_track_maps.cloud_folder_id = ?
            ) ORDER BY created_at DESC"#,
        )
        .bind(folder_id)
    } else {
        UploadQueueItem::query("SELECT * FROM upload_queue ORDER BY created_at DESC")
    };

    let upload_items = upload_query.fetch_all(&mut db.connection).await?;

    // Get download queue items
    let download_query = if let Some(folder_id) = &folder_id {
        DownloadQueueItem::query(
            r#"SELECT * FROM download_queue WHERE download_queue.cloud_track_map_id IN (
                SELECT cloud_track_maps.id FROM cloud_track_maps WHERE cloud_track_maps.cloud_folder_id = ?
            ) ORDER BY created_at DESC"#,
        )
        .bind(folder_id)
    } else {
        DownloadQueueItem::query("SELECT * FROM download_queue ORDER BY created_at DESC")
    };

    let download_items = download_query.fetch_all(&mut db.connection).await?;

    // Convert upload items to DTOs
    for item in upload_items {
        // Get track info
        let track_map = CloudTrackMap::select()
            .where_("id = ?")
            .bind(&item.cloud_track_map_id)
            .fetch_optional(&mut db.connection)
            .await?;

        let track = if let Some(map) = track_map {
            CloudTrack::select()
                .where_("id = ?")
                .bind(&map.cloud_track_id)
                .fetch_optional(&mut db.connection)
                .await?
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
        let track_map = CloudTrackMap::select()
            .where_("id = ?")
            .bind(&item.cloud_track_map_id)
            .fetch_optional(&mut db.connection)
            .await?;

        let track = if let Some(map) = track_map {
            CloudTrack::select()
                .where_("id = ?")
                .bind(&map.cloud_track_id)
                .fetch_optional(&mut db.connection)
                .await?
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

    // Get all queue items and filter by folder if specified
    let mut download_items = if let Some(folder_id) = &folder_id {
        DownloadQueueItem::query(
            "SELECT q.* FROM download_queue q 
            JOIN cloud_track_maps m ON q.cloud_track_map_id = m.id 
            WHERE m.cloud_folder_id = ?",
        )
        .bind(folder_id)
        .fetch_all(&mut db.connection)
        .await?
    } else {
        DownloadQueueItem::query("SELECT * FROM download_queue")
            .fetch_all(&mut db.connection)
            .await?
    };

    let mut upload_items = if let Some(folder_id) = &folder_id {
        UploadQueueItem::query(
            "SELECT q.* FROM upload_queue q 
            JOIN cloud_track_maps m ON q.cloud_track_map_id = m.id 
            WHERE m.cloud_folder_id = ?",
        )
        .bind(folder_id)
        .fetch_all(&mut db.connection)
        .await?
    } else {
        UploadQueueItem::query("SELECT * FROM upload_queue")
            .fetch_all(&mut db.connection)
            .await?
    };

    // Count items in each state
    let mut pending_count = 0;
    let mut in_progress_count = 0;
    let mut completed_count = 0;
    let mut failed_count = 0;

    // Count download queue items
    for item in download_items {
        let status = SyncStatus::from_str(&item.status)?;
        match status {
            SyncStatus::Pending => pending_count += 1,
            SyncStatus::InProgress => in_progress_count += 1,
            SyncStatus::Completed => completed_count += 1,
            SyncStatus::Failed { .. } => failed_count += 1,
        }
    }

    // Count upload queue items
    for item in upload_items {
        let status = SyncStatus::from_str(&item.status)?;
        match status {
            SyncStatus::Pending => pending_count += 1,
            SyncStatus::InProgress => in_progress_count += 1,
            SyncStatus::Completed => completed_count += 1,
            SyncStatus::Failed { .. } => failed_count += 1,
        }
    }

    Ok(QueueStatsDTO {
        pending_count,
        in_progress_count,
        completed_count,
        failed_count,
    })
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
    let folder = CloudFolder::select()
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
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    let mut db = db_state.get_lock().await;
    let now = Utc::now();

    for track_id in track_ids {
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

        // Get folder to determine provider type
        let folder = CloudFolder::select()
            .where_("id = ?")
            .bind(&track_map.cloud_folder_id)
            .fetch_one(&mut db.connection)
            .await?;

        // Create upload queue item
        let upload_item = UploadQueueItem {
            id: Uuid::new_v4().to_string(),
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
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    let mut db = db_state.get_lock().await;
    let now = Utc::now();

    for track_id in track_ids {
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

        // Get folder to determine provider type
        let folder = CloudFolder::select()
            .where_("id = ?")
            .bind(&track_map.cloud_folder_id)
            .fetch_one(&mut db.connection)
            .await?;

        // Create download queue item
        let download_item = DownloadQueueItem {
            id: Uuid::new_v4().to_string(),
            cloud_track_map_id: track_map.id,
            provider_type: folder.provider_type,
            status: "pending".to_string(),
            error_message: None,
            created_at: now,
            updated_at: now,
            attempts: 0,
        };

        download_item.insert(&mut db.connection).await?;
    }

    Ok(())
}