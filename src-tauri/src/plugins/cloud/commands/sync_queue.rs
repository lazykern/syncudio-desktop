use chrono::Utc;
use log::info;
use ormlite::Model;
use serde::Deserialize;
use serde::Serialize;
use tauri::Emitter;
use tauri::Runtime;
use ts_rs::TS;
use std::path::Path;
use std::path::PathBuf;
use tauri::{Manager, State};
use uuid::Uuid;


use crate::libs::error::SyncudioError;
use crate::libs::track::{self, Track};
use crate::plugins::cloud::CloudProvider;
use crate::plugins::cloud::CloudProviderType;
use crate::plugins::cloud::CloudState;
use crate::plugins::cloud::models::*;
use crate::plugins::cloud::models::dto::*;
use crate::plugins::db::DBState;
use crate::libs::error::AnyResult;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
struct TrackDownloadedPayload {
    track_id: String,
    location_type: String,
    local_track_id: String,
    cloud_track_id: String,
    sync_folder_id: String,
    relative_path: String,
}

#[derive(Debug, ormlite::FromRow)]
struct QueueStatsRow {
    status: String,
    count: i32,
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
            r#"SELECT u.* 
            FROM upload_queue u
            INNER JOIN cloud_maps m ON u.cloud_map_id = m.id
            WHERE m.cloud_music_folder_id = ?
            ORDER BY u.created_at ASC"#,
        )
        .bind(folder_id)
    } else {
        UploadQueueItem::query("SELECT * FROM upload_queue ORDER BY created_at ASC")
    };

    let upload_items = upload_query.fetch_all(&mut db.connection).await?;

    // Get download queue items
    let download_query = if let Some(folder_id) = &folder_id {
        DownloadQueueItem::query(
            r#"SELECT d.* 
            FROM download_queue d
            INNER JOIN cloud_maps m ON d.cloud_map_id = m.id
            WHERE m.cloud_music_folder_id = ?
            ORDER BY d.created_at ASC"#,
        )
        .bind(folder_id)
    } else {
        DownloadQueueItem::query("SELECT * FROM download_queue ORDER BY created_at ASC")
    };

    let download_items = download_query.fetch_all(&mut db.connection).await?;

    // Convert upload items to DTOs
    for item in upload_items {
        // Get track info through track map
        let track_map = CloudTrackMap::select()
            .where_("id = ?")
            .bind(&item.cloud_map_id)
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
        // Get track info through track map
        let track_map = CloudTrackMap::select()
            .where_("id = ?")
            .bind(&item.cloud_map_id)
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
                SELECT u.status 
                FROM upload_queue u
                INNER JOIN cloud_maps m ON u.cloud_map_id = m.id
                WHERE m.cloud_music_folder_id = ?
                UNION ALL
                SELECT d.status 
                FROM download_queue d
                INNER JOIN cloud_maps m ON d.cloud_map_id = m.id
                WHERE m.cloud_music_folder_id = ?
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

        // Get folder to check file existence
        let folder = CloudMusicFolder::select()
            .where_("id = ?")
            .bind(&track_map.cloud_music_folder_id)
            .fetch_one(&mut db.connection)
            .await?;

        // Check if file exists locally
        let local_path = Path::new(&folder.local_folder_path)
            .join(&track_map.relative_path)
            .to_string_lossy()
            .to_string();

        if !Path::new(&local_path).exists() {
            return Err(SyncudioError::FileNotFound(local_path));
        }

        // Create upload queue item
        let item = UploadQueueItem {
            id: Uuid::new_v4().to_string(),
            cloud_map_id: track_map.id,
            provider_type: folder.provider_type,
            priority: priority.unwrap_or(0),
            status: "pending".to_string(),
            error_message: None,
            created_at: now,
            updated_at: now,
            attempts: 0,
        };

        item.insert(&mut db.connection).await?;
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
            .where_("cloud_map_id = ? AND (status = 'pending' OR status = 'in_progress')")
            .bind(&track_map.id)
            .fetch_optional(&mut db.connection)
            .await?
            .is_some();

        let has_active_download = DownloadQueueItem::select()
            .where_("cloud_map_id = ? AND (status = 'pending' OR status = 'in_progress')")
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
        let folder = CloudMusicFolder::select()
            .where_("id = ?")
            .bind(&track_map.cloud_music_folder_id)
            .fetch_one(&mut db.connection)
            .await?;

        // Create download queue item
        let download_item = DownloadQueueItem {
            id: Uuid::new_v4().to_string(),
            priority: priority.unwrap_or(0),
            cloud_map_id: track_map.id,
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
            AND cloud_map_id IN (
                SELECT id FROM cloud_maps WHERE cloud_music_folder_id = ?
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
            AND cloud_map_id IN (
                SELECT id FROM cloud_maps WHERE cloud_music_folder_id = ?
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
    // First database operation block - get required info
    let (track_map, track, folder, mut item) = {
        let mut db = db_state.get_lock().await;
        
        // Get queue item
        let item = UploadQueueItem::select()
            .where_("id = ?")
            .bind(&item_id)
            .fetch_one(&mut db.connection)
            .await?;

        // Get track info
        let track_map = CloudTrackMap::select()
            .where_("id = ?")
            .bind(&item.cloud_map_id)
            .fetch_one(&mut db.connection)
            .await?;

        let track = CloudTrack::select()
            .where_("id = ?")
            .bind(&track_map.cloud_track_id)
            .fetch_one(&mut db.connection)
            .await?;

        // Get folder info
        let folder = CloudMusicFolder::select()
            .where_("id = ?")
            .bind(&track_map.cloud_music_folder_id)
            .fetch_one(&mut db.connection)
            .await?;

        (track_map, track, folder, item)
    };

    // Get local file path - No database lock needed
    let local_path = Path::new(&folder.local_folder_path)
        .join(&track_map.relative_path)
        .to_string_lossy()
        .to_string();

    if !Path::new(&local_path).exists() {
        return Err(SyncudioError::FileNotFound(local_path));
    }

    // Update item status to in_progress
    {
        let mut db = db_state.get_lock().await;
        item.status = "in_progress".to_string();
        item.updated_at = Utc::now();
        item = item.update_all_fields(&mut db.connection).await?;
    }

    // Get cloud provider - No database lock needed
    let provider = match folder.provider_type.as_str() {
        "dropbox" => &cloud_state.dropbox,
        _ => return Err(SyncudioError::UnsupportedProvider(folder.provider_type)),
    };

    // Upload file - No database lock needed
    let cloud_file = provider
        .upload_file(
            &PathBuf::from(&local_path),
            &track_map.relative_path,
            Some(&folder.cloud_folder_path),
        )
        .await?;

    // Final database operation block - update track map and complete the operation
    {
        let mut db = db_state.get_lock().await;

        // Update track map with cloud file ID
        let mut updated_map = track_map;
        updated_map.cloud_file_id = Some(cloud_file.id.clone());
        updated_map.update_all_fields(&mut db.connection).await?;

        // Update track metadata
        let mut updated_track = track;
        updated_track.updated_at = Utc::now();
        updated_track.update_all_fields(&mut db.connection).await?;

        // Mark item as completed
        item.status = "completed".to_string();
        item.updated_at = Utc::now();
        item.update_all_fields(&mut db.connection).await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn start_download<R: Runtime>(
    app: tauri::AppHandle<R>,
    item_id: String,
    db_state: State<'_, DBState>,
    cloud_state: State<'_, CloudState>,
) -> AnyResult<()> {
    // First database operation block - get required info
    let (track_map, track, folder, mut item) = {
        let mut db = db_state.get_lock().await;
        
        // Get queue item
        let item = DownloadQueueItem::select()
            .where_("id = ?")
            .bind(&item_id)
            .fetch_one(&mut db.connection)
            .await?;

        // Get track info
        let track_map = CloudTrackMap::select()
            .where_("id = ?")
            .bind(&item.cloud_map_id)
            .fetch_one(&mut db.connection)
            .await?;

        let track = CloudTrack::select()
            .where_("id = ?")
            .bind(&track_map.cloud_track_id)
            .fetch_one(&mut db.connection)
            .await?;

        // Get folder info
        let folder = CloudMusicFolder::select()
            .where_("id = ?")
            .bind(&track_map.cloud_music_folder_id)
            .fetch_one(&mut db.connection)
            .await?;

        (track_map, track, folder, item)
    };

    // Get local file path
    let local_path = Path::new(&folder.local_folder_path)
        .join(&track_map.relative_path)
        .to_string_lossy()
        .to_string();

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(&local_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Update item status to in_progress
    {
        let mut db = db_state.get_lock().await;
        item.status = "in_progress".to_string();
        item.updated_at = Utc::now();
        item = item.update_all_fields(&mut db.connection).await?;
    }

    // Get cloud provider
    let provider = match folder.provider_type.as_str() {
        "dropbox" => &cloud_state.dropbox,
        _ => return Err(SyncudioError::UnsupportedProvider(folder.provider_type)),
    };

    // Download file - No database lock needed here
    provider
        .download_file(&track_map.cloud_file_id.clone().unwrap(), &PathBuf::from(&local_path))
        .await?;

    // Parse local track metadata - No database lock needed
    let mut local_track = track::get_track_from_file(&PathBuf::from(&local_path))
        .ok_or_else(|| SyncudioError::InvalidTrackMetadata(local_path.clone()))?;

    // Final database operation block - update track and complete the operation
    let (local_track, track) = {
        let mut db = db_state.get_lock().await;

        // Check if track already exists and update it instead of inserting
        let existing_track = Track::select()
            .where_("path = ?")
            .bind(&local_track.path)
            .fetch_optional(&mut db.connection)
            .await?;

        let local_track = if let Some(existing) = existing_track {
            // Update existing track with new metadata
            local_track.id = existing.id; // Keep the same ID
            local_track.update_all_fields(&mut db.connection).await?
        } else {
            // Insert as new track if it doesn't exist
            local_track.insert(&mut db.connection).await?
        };

        // Update cloud track metadata
        let mut track = track;
        track.updated_at = Utc::now();
        track.tags = Some(CloudTrackTag::from_track(local_track.clone()));
        let track = track.update_all_fields(&mut db.connection).await?;

        // Mark item as completed
        item.status = "completed".to_string();
        item.updated_at = Utc::now();
        item.update_all_fields(&mut db.connection).await?;

        (local_track, track)
    };

    // Emit track downloaded event - No database lock needed
    app.emit(
        "track-downloaded",
        TrackDownloadedPayload {
            track_id: track.id.clone(),
            location_type: "both".to_string(),
            local_track_id: local_track.id.clone(),
            cloud_track_id: track.id.clone(),
            sync_folder_id: folder.id.clone(),
            relative_path: track_map.relative_path,
        },
    )?;

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
