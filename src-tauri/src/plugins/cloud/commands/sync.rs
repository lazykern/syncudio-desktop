use uuid::Uuid;
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
                query_models::{TrackWithMapRow, QueueOperationRow, QueueStatsRow},
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
    let folder = CloudFolder::select()
        .where_("id = ?")
        .bind(&folder_id)
        .fetch_one(&mut db.connection)
        .await?;

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
    let upload_query = if let Some(folder_id) = &folder_id {
        UploadQueueItem::query(
            r#"SELECT * FROM upload_queue WHERE upload_queue.cloud_track_map_id IN (
                SELECT cloud_track_maps.id FROM cloud_track_maps WHERE cloud_track_maps.cloud_folder_id = ?
            ) ORDER BY created_at ASC"#,
        )
        .bind(folder_id)
    } else {
        UploadQueueItem::query("SELECT * FROM upload_queue ORDER BY created_at ASC")
    };

    let upload_items = upload_query.fetch_all(&mut db.connection).await?;

    // Get download queue items
    let download_query = if let Some(folder_id) = &folder_id {
        DownloadQueueItem::query(
            r#"SELECT * FROM download_queue WHERE download_queue.cloud_track_map_id IN (
                SELECT cloud_track_maps.id FROM cloud_track_maps WHERE cloud_track_maps.cloud_folder_id = ?
            ) ORDER BY created_at ASC"#,
        )
        .bind(folder_id)
    } else {
        DownloadQueueItem::query("SELECT * FROM download_queue ORDER BY created_at ASC")
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

    let stats: Vec<QueueStatsRow> = if let Some(folder_id) = folder_id {
        ormlite::query_as(r#"
            SELECT status, COUNT(*) as count
            FROM (
                SELECT status FROM upload_queue u
                INNER JOIN cloud_track_maps m ON u.cloud_track_map_id = m.id
                WHERE m.cloud_folder_id = ?
                UNION ALL
                SELECT status FROM download_queue d
                INNER JOIN cloud_track_maps m ON d.cloud_track_map_id = m.id
                WHERE m.cloud_folder_id = ?
            ) combined
            GROUP BY status
        "#)
        .bind(&folder_id)
        .bind(&folder_id)
        .fetch_all(&mut db.connection)
        .await?
    } else {
        ormlite::query_as(r#"
            SELECT status, COUNT(*) as count
            FROM (
                SELECT status FROM upload_queue
                UNION ALL
                SELECT status FROM download_queue
            ) combined
            GROUP BY status
        "#)
            .fetch_all(&mut db.connection)
            .await?
    };

    let mut pending_count = 0;
    let mut in_progress_count = 0;
    let mut completed_count = 0;
    let mut failed_count = 0;

    for stat in stats {
        match stat.status.as_str() {
            "pending" => pending_count = stat.count,
            "in_progress" => in_progress_count = stat.count,
            "completed" => completed_count = stat.count,
            _ => failed_count += stat.count,
        }
    }

    Ok(QueueStatsDTO {
        pending_count,
        in_progress_count,
        completed_count,
        failed_count,
    })
}

/// Command to retry failed sync items
#[tauri::command]
pub async fn retry_failed_items(
    folder_id: Option<String>,
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    let mut db = db_state.get_lock().await;

    // Get failed upload items
    let upload_query = if let Some(folder_id) = &folder_id {
        UploadQueueItem::query(
            r#"SELECT * FROM upload_queue 
            WHERE status = 'failed' 
            AND cloud_track_map_id IN (
                SELECT id FROM cloud_track_maps WHERE cloud_folder_id = ?
            )"#,
        )
        .bind(folder_id)
    } else {
        UploadQueueItem::query("SELECT * FROM upload_queue WHERE status = 'failed'")
    };

    let upload_items = upload_query.fetch_all(&mut db.connection).await?;

    // Get failed download items
    let download_query = if let Some(folder_id) = &folder_id {
        DownloadQueueItem::query(
            r#"SELECT * FROM download_queue 
            WHERE status = 'failed' 
            AND cloud_track_map_id IN (
                SELECT id FROM cloud_track_maps WHERE cloud_folder_id = ?
            )"#,
        )
        .bind(folder_id)
    } else {
        DownloadQueueItem::query("SELECT * FROM download_queue WHERE status = 'failed'")
    };

    let download_items = download_query.fetch_all(&mut db.connection).await?;

    // Reset failed items to pending
    for mut item in upload_items {
        item.retry();
        item.update_all_fields(&mut db.connection).await?;
    }

    for mut item in download_items {
        item.retry();
        item.update_all_fields(&mut db.connection).await?;
    }

    Ok(())
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
    priority: Option<i32>,
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

        // Check for existing active operations
        let has_active_upload = UploadQueueItem::select()
            .where_("cloud_track_map_id = ? AND (status = 'pending' OR status = 'in_progress')")
            .bind(&track_map.id)
            .fetch_optional(&mut db.connection)
            .await?
            .is_some();

        let has_active_download = DownloadQueueItem::select()
            .where_("cloud_track_map_id = ? AND (status = 'pending' OR status = 'in_progress')")
            .bind(&track_map.id)
            .fetch_optional(&mut db.connection)
            .await?
            .is_some();

        if has_active_upload || has_active_download {
            // Skip this track as it already has an active operation
            info!("Skipping track {} as it already has an active sync operation", track_id);
            continue;
        }

        // Get folder to determine provider type
        let folder = CloudFolder::select()
            .where_("id = ?")
            .bind(&track_map.cloud_folder_id)
            .fetch_one(&mut db.connection)
            .await?;

        // Create upload queue item
        let upload_item = UploadQueueItem {
            id: Uuid::new_v4().to_string(),
            priority: 0,
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
    priority: Option<i32>,
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

        // Check for existing active operations
        let has_active_upload = UploadQueueItem::select()
            .where_("cloud_track_map_id = ? AND (status = 'pending' OR status = 'in_progress')")
            .bind(&track_map.id)
            .fetch_optional(&mut db.connection)
            .await?
            .is_some();

        let has_active_download = DownloadQueueItem::select()
            .where_("cloud_track_map_id = ? AND (status = 'pending' OR status = 'in_progress')")
            .bind(&track_map.id)
            .fetch_optional(&mut db.connection)
            .await?
            .is_some();

        if has_active_upload || has_active_download {
            // Skip this track as it already has an active operation
            info!("Skipping track {} as it already has an active sync operation", track_id);
            continue;
        }

        // Get folder to determine provider type
        let folder = CloudFolder::select()
            .where_("id = ?")
            .bind(&track_map.cloud_folder_id)
            .fetch_one(&mut db.connection)
            .await?;

        // Create download queue item
        let download_item = DownloadQueueItem {
            id: Uuid::new_v4().to_string(),
            priority: 0,
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

#[tauri::command]
pub async fn reset_in_progress_items(
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    info!("Resetting in-progress items to pending state");
    let mut db = db_state.get_lock().await;

    // Reset any items stuck in "in_progress" state back to "pending"
    let upload_items = UploadQueueItem::select()
        .where_("status = 'in_progress'")
        .fetch_all(&mut db.connection)
        .await?;

    info!("Found {} upload items in progress", upload_items.len());
    for mut item in upload_items {
        info!("Resetting upload item {} to pending", item.id);
        item.retry();
        item.update_all_fields(&mut db.connection).await?;
    }

    let download_items = DownloadQueueItem::select()
        .where_("status = 'in_progress'")
        .fetch_all(&mut db.connection)
        .await?;

    info!("Found {} download items in progress", download_items.len());
    for mut item in download_items {
        info!("Resetting download item {} to pending", item.id);
        item.retry();
        item.update_all_fields(&mut db.connection).await?;
    }

    info!("Successfully reset all in-progress items");
    Ok(())
}

#[tauri::command]
pub async fn get_next_upload_item(
    db_state: State<'_, DBState>,
) -> AnyResult<Option<UploadQueueItem>> {
    let mut db = db_state.get_lock().await;

    let item = UploadQueueItem::query("SELECT * FROM upload_queue WHERE status = 'pending' ORDER BY created_at ASC LIMIT 1")
        .fetch_optional(&mut db.connection)
        .await?;

    if let Some(ref item) = item {
        info!("Found next upload item: {}", item.id);
    } 
    Ok(item)
}

#[tauri::command]
pub async fn get_next_download_item(
    db_state: State<'_, DBState>,
) -> AnyResult<Option<DownloadQueueItem>> {
    let mut db = db_state.get_lock().await;

    let item = DownloadQueueItem::query("SELECT * FROM download_queue WHERE status = 'pending' ORDER BY created_at ASC LIMIT 1")
        .fetch_optional(&mut db.connection)
        .await?;

    if let Some(ref item) = item {
        info!("Found next download item: {}", item.id);
    }

    Ok(item)
}

#[tauri::command]
pub async fn start_upload(
    item_id: String,
    db_state: State<'_, DBState>,
    cloud_state: State<'_, CloudState>,
) -> AnyResult<()> {
    info!("Starting upload for item: {}", item_id);
    // Get all necessary data under a short-lived lock
    let (item, track_map, cloud_folder, local_path) = {
        let mut db = db_state.get_lock().await;

        let item = UploadQueueItem::select()
            .where_("id = ?")
            .bind(&item_id)
            .fetch_one(&mut db.connection)
            .await?;

        let track_map = CloudTrackMap::select()
            .where_("id = ?")
            .bind(&item.cloud_track_map_id)
            .fetch_one(&mut db.connection)
            .await?;

        let cloud_folder = CloudFolder::select()
            .where_("id = ?")
            .bind(&track_map.cloud_folder_id)
            .fetch_one(&mut db.connection)
            .await?;

        let local_path = Path::new(&cloud_folder.local_folder_path)
            .join(&track_map.relative_path)
            .to_string_lossy()
            .to_string();

        info!("Updating item {} status to in_progress", item_id);
        // Update status to in_progress
        let mut item_to_update = item.clone();
        item_to_update.start_processing();
        item_to_update = item_to_update.update_all_fields(&mut db.connection).await?;

        (item_to_update, track_map, cloud_folder, local_path)
    }; // Lock is released here

    if !Path::new(&local_path).exists() {
        info!("File not found at {}", local_path);
        return Err(SyncudioError::Path(format!("File not found at {}", local_path)));
    }

    // Perform the upload without holding the lock
    {
        let cloud_file = match CloudProviderType::from_str(&cloud_folder.provider_type)? {
            CloudProviderType::Dropbox => {
                info!("Starting Dropbox upload for {}", local_path);
                // get parent path and file name
                let cloud_path = Path::new(&cloud_folder.cloud_folder_path).join(&track_map.relative_path);
                let file_name = cloud_path.file_name().unwrap().to_string_lossy().to_string();
                let parent_path = cloud_path.parent().unwrap().to_string_lossy().to_string();
                let cloud_file = cloud_state.dropbox.upload_file(&PathBuf::from(&local_path), &file_name, Some(&parent_path)).await?;
                info!("Successfully uploaded file to Dropbox: {}", cloud_path.display());
                cloud_file
            }
            CloudProviderType::GoogleDrive => {
                info!("Google Drive upload not implemented");
                return Err(SyncudioError::GoogleDrive("Google Drive not implemented yet".to_string()));
            }
        };

        // Re-acquire lock briefly to update status and cloud file ID
        let mut db = db_state.get_lock().await;
        let mut item_to_update = item;
        item_to_update.complete();
        info!("Marking upload item {} as completed", item_id);
        item_to_update.update_all_fields(&mut db.connection).await?;

        let mut cloud_track = CloudTrack::select()
            .where_("id = ?")
            .bind(&track_map.cloud_track_id)
            .fetch_one(&mut db.connection)
            .await?;
        cloud_track.cloud_file_id = Some(cloud_file.id);
        cloud_track.update_all_fields(&mut db.connection).await?;
    }

    info!("Upload completed successfully for item: {}", item_id);
    Ok(())
}

#[tauri::command]
pub async fn start_download(
    item_id: String,
    db_state: State<'_, DBState>,
    cloud_state: State<'_, CloudState>,
) -> AnyResult<()> {
    info!("Starting download for item: {}", item_id);
    // Get all necessary data under a short-lived lock
    let (item, track_map, cloud_track, cloud_folder, local_path) = {
        let mut db = db_state.get_lock().await;

        let item = DownloadQueueItem::select()
            .where_("id = ?")
            .bind(&item_id)
            .fetch_one(&mut db.connection)
            .await?;

        let track_map = CloudTrackMap::select()
            .where_("id = ?")
            .bind(&item.cloud_track_map_id)
            .fetch_one(&mut db.connection)
            .await?;

        let cloud_track = CloudTrack::select()
            .where_("id = ? AND cloud_file_id IS NOT NULL")
            .bind(&track_map.cloud_track_id)
            .fetch_one(&mut db.connection)
            .await?;

        let cloud_folder = CloudFolder::select()
            .where_("id = ?")
            .bind(&track_map.cloud_folder_id)
            .fetch_one(&mut db.connection)
            .await?;

        let local_path = Path::new(&cloud_folder.local_folder_path)
            .join(&track_map.relative_path)
            .to_string_lossy()
            .to_string();

        info!("Updating item {} status to in_progress", item_id);
        // Update status to in_progress
        let mut item_to_update = item.clone();
        item_to_update.start_processing();
        item_to_update = item_to_update.update_all_fields(&mut db.connection).await?;

        (item_to_update, track_map, cloud_track, cloud_folder, local_path)
    }; // Lock is released here

    // Create parent directory if it doesn't exist
    if let Some(parent) = Path::new(&local_path).parent() {
        info!("Creating parent directory: {}", parent.display());
        std::fs::create_dir_all(parent)?;
    }

    // Perform the download without holding the lock
    match CloudProviderType::from_str(&cloud_folder.provider_type)? {
        CloudProviderType::Dropbox => {
            info!("Starting Dropbox download to {}", local_path);
            cloud_state.dropbox.download_file(&cloud_track.cloud_file_id.unwrap(), &PathBuf::from(&local_path)).await?;
            info!("Successfully downloaded file from Dropbox");
        }
        CloudProviderType::GoogleDrive => {
            info!("Google Drive download not implemented");
            return Err(SyncudioError::GoogleDrive("Google Drive not implemented yet".to_string()));
        }
    }

    // Re-acquire lock briefly to update status
    {
        let mut db = db_state.get_lock().await;
        let mut item_to_update = item;
        item_to_update.complete();
        info!("Marking download item {} as completed", item_id);
        item_to_update.update_all_fields(&mut db.connection).await?;

        let mut cloud_track = CloudTrack::select()
            .where_("id = ?")
            .bind(&track_map.cloud_track_id)
            .fetch_one(&mut db.connection)
            .await?;
        cloud_track.blake3_hash = blake3_hash(&PathBuf::from(&local_path)).ok();
        cloud_track.update_all_fields(&mut db.connection).await?;
    }

    info!("Download completed successfully for item: {}", item_id);
    Ok(())
}

#[tauri::command]
pub async fn fail_upload(
    item_id: String,
    error: String,
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    info!("Marking upload item {} as failed: {}", item_id, error);
    let mut db = db_state.get_lock().await;

    let mut item = UploadQueueItem::select()
        .where_("id = ?")
        .bind(&item_id)
        .fetch_one(&mut db.connection)
        .await?;

    item.fail(error);
    item.update_all_fields(&mut db.connection).await?;
    info!("Upload item {} marked as failed", item_id);

    Ok(())
}

#[tauri::command]
pub async fn fail_download(
    item_id: String,
    error: String,
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    info!("Marking download item {} as failed: {}", item_id, error);
    let mut db = db_state.get_lock().await;

    let mut item = DownloadQueueItem::select()
        .where_("id = ?")
        .bind(&item_id)
        .fetch_one(&mut db.connection)
        .await?;

    item.fail(error);
    item.update_all_fields(&mut db.connection).await?;
    info!("Download item {} marked as failed", item_id);

    Ok(())
}