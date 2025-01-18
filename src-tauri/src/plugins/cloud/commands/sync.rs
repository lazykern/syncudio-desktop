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
                    TrackSyncDetailsDTO,
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
    todo!("Implement get_queue_items");
    // 1. Get active items from upload/download queues
    // 2. Filter by folder if specified
    // 3. Return queue items
}

/// Get queue statistics
#[tauri::command]
pub async fn get_queue_stats(
    folder_id: Option<String>,
    db_state: State<'_, DBState>,
) -> AnyResult<QueueStatsDTO> {
    todo!("Implement get_queue_stats");
    // 1. Count items in each state
    // 2. Filter by folder if specified
    // 3. Return stats
}

/// Command to force sync a folder
#[tauri::command]
pub async fn force_sync_folder(
    folder_id: String,
    db_state: State<'_, DBState>,
    cloud_state: State<'_, CloudState>,
) -> AnyResult<()> {
    todo!("Implement force_sync_folder");
    // 1. Check folder exists
    // 2. Queue all out of sync tracks
    // 3. Start sync process
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

/// Command to get detailed sync status for a track
#[tauri::command]
pub async fn get_track_sync_details(
    track_id: String,
    db_state: State<'_, DBState>,
) -> AnyResult<TrackSyncDetailsDTO> {
    todo!("Implement get_track_sync_details");
    // 1. Get track info
    // 2. Get sync history
    // 3. Get current queue status
    // 4. Return detailed status
}
