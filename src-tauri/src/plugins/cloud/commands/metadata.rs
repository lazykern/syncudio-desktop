use crate::libs::error::AnyResult;
use crate::plugins::cloud::models::{
    CloudMetadataCollection, CloudTrack, CloudTrackFullDTO, CloudTrackMap, CloudTrackMetadata,
};
use crate::plugins::cloud::{CloudMetadataSyncResult, CloudMetadataUpdateResult, CloudProvider, CloudState};
use crate::plugins::db::DBState;
use chrono::Utc;
use log::info;
use ormlite::Model;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;
use std::env::temp_dir;
use chrono::DateTime;

#[tauri::command]
pub async fn pull_cloud_metadata(
    db_state: State<'_, DBState>,
    cloud_state: State<'_, CloudState>,
) -> AnyResult<CloudMetadataSyncResult> {
    info!("Starting cloud metadata sync");
    let provider = &cloud_state.dropbox;

    // 1. Download metadata from cloud - No DB lock needed
    let metadata_folder = provider.ensure_metadata_folder().await?;
    info!("Ensured metadata folder exists");
    
    let temp_path = temp_dir().join("syncudio_tracks.json.tmp");
    info!("Using temp file: {}", temp_path.display());
    
    let (cloud_metadata, is_fresh_start) = match provider.download_file("/Syncudio/metadata/tracks.json", &temp_path).await {
        Ok(_) => {
            info!("Successfully downloaded metadata file");
            let content = std::fs::read_to_string(&temp_path)?;
            info!("Read metadata file content: {} bytes", content.len());
            
            let metadata: CloudMetadataCollection = serde_json::from_str(&content)?;
            info!("Parsed metadata with {} tracks", metadata.tracks.len());
            
            std::fs::remove_file(&temp_path)?;
            info!("Cleaned up temp file");
            
            (metadata, false)
        }
        Err(e) => {
            info!("No existing metadata found, starting fresh: {}", e);
            (CloudMetadataCollection::new(), true)
        }
    };

    let mut result = CloudMetadataSyncResult::new(is_fresh_start);

    // 2. Load database state with minimal lock time
    info!("Loading database state");
    let db_tracks = {
        let mut db = db_state.get_lock().await;
        let tracks = db.get_cloud_tracks_full_by_provider(provider.provider_type().as_str()).await?;
        info!("Loaded {} tracks from database", tracks.len());
        tracks
    };

    // Create lookup maps - No DB lock needed
    let mut db_tracks_by_path: HashMap<String, CloudTrackFullDTO> = HashMap::new();
    let mut db_tracks_by_cloud_id: HashMap<String, CloudTrackFullDTO> = HashMap::new();

    for track in &db_tracks {
        db_tracks_by_path.insert(track.relative_path.clone(), track.clone());
        if let Some(cloud_id) = &track.cloud_file_id {
            db_tracks_by_cloud_id.insert(cloud_id.clone(), track.clone());
        }
    }
    info!("Created lookup maps: {} by path, {} by cloud ID", 
          db_tracks_by_path.len(), 
          db_tracks_by_cloud_id.len());

    // Process cloud metadata tracks in batches to minimize lock time
    for cloud_track in &cloud_metadata.tracks {
        let db_track = db_tracks_by_path
            .get(&cloud_track.relative_path)
            .or_else(|| db_tracks_by_cloud_id.get(&cloud_track.cloud_file_id));

        match db_track {
            Some(track) => {
                // Convert last_modified from string timestamp to DateTime
                let cloud_modified = cloud_track.last_modified.parse::<i64>()
                    .map(|ts| DateTime::from_timestamp(ts / 1000, 0).unwrap_or_default())
                    .unwrap_or_default();

                if cloud_modified > track.track_updated_at {
                    // Update with minimal lock time
                    let mut db = db_state.get_lock().await;
                    
                    info!("Updating track {} from cloud metadata (path: {})", 
                          track.track_id, 
                          cloud_track.relative_path);
                    
                    let mut updated_track = CloudTrack::select()
                        .where_("id = ?")
                        .bind(&track.track_id)
                        .fetch_one(&mut db.connection)
                        .await?;

                    updated_track.tags = cloud_track.tags.clone();
                    updated_track.updated_at = cloud_modified;
                    updated_track.update_all_fields(&mut db.connection).await?;

                    // Update map if cloud_file_id changed
                    if let Some(map) = CloudTrackMap::select()
                        .where_("cloud_track_id = ?")
                        .bind(&track.track_id)
                        .fetch_optional(&mut db.connection)
                        .await?
                    {
                        if map.cloud_file_id.as_ref() != Some(&cloud_track.cloud_file_id) {
                            info!("Updating cloud file ID for track {}: {} -> {}", 
                                  track.track_id,
                                  map.cloud_file_id.as_deref().unwrap_or("none"),
                                  cloud_track.cloud_file_id);
                                  
                            let mut updated_map = map;
                            updated_map.cloud_file_id = Some(cloud_track.cloud_file_id.clone());
                            updated_map.update_all_fields(&mut db.connection).await?;
                        }
                    }
                    result.tracks_updated += 1;
                } else {
                    info!("Skipping track {} (not modified)", track.track_id);
                }
            }
            None => {
                // Create new entry with minimal lock time
                let mut db = db_state.get_lock().await;
                
                info!("Creating new track from cloud metadata (path: {})", 
                      cloud_track.relative_path);
                
                // Convert last_modified from string timestamp to DateTime
                let cloud_modified = cloud_track.last_modified.parse::<i64>()
                    .map(|ts| DateTime::from_timestamp(ts / 1000, 0).unwrap_or_default())
                    .unwrap_or_default();

                let track = CloudTrack {
                    id: Uuid::new_v4().to_string(),
                    file_name: Path::new(&cloud_track.cloud_path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    size: cloud_track.size,
                    updated_at: cloud_modified,
                    tags: cloud_track.tags.clone(),
                };
                let track_id = track.id.clone();
                track.insert(&mut db.connection).await?;

                // Create map
                let map = CloudTrackMap {
                    id: Uuid::new_v4().to_string(),
                    cloud_track_id: track_id.clone(),
                    cloud_music_folder_id: cloud_track.cloud_folder_id.clone(),
                    relative_path: cloud_track.relative_path.clone(),
                    cloud_file_id: Some(cloud_track.cloud_file_id.clone()),
                };
                map.insert(&mut db.connection).await?;
                result.tracks_created += 1;
            }
        }
    }

    info!("Metadata sync completed: {} tracks updated, {} tracks created", 
          result.tracks_updated, 
          result.tracks_created);

    Ok(result)
}

#[tauri::command]
pub async fn push_cloud_metadata(
    db_state: State<'_, DBState>,
    cloud_state: State<'_, CloudState>,
) -> AnyResult<CloudMetadataUpdateResult> {
    info!("Updating cloud metadata");
    let provider = &cloud_state.dropbox;
    let mut result = CloudMetadataUpdateResult::new();

    // 1. Get current database state with minimal lock time
    let tracks = {
        let mut db = db_state.get_lock().await;
        db.get_cloud_tracks_full_by_provider(provider.provider_type().as_str()).await?
    };

    // 2. Convert to cloud metadata format - No DB lock needed
    let metadata = CloudMetadataCollection {
        tracks: tracks
            .into_iter()
            .filter_map(|t| {
                let cloud_path = t.cloud_path();
                // Only include tracks that have both cloud_file_id and valid tags
                match (t.cloud_file_id, t.tags) {
                    (Some(cloud_id), Some(tags)) => {
                        result.tracks_included += 1;
                        Some(CloudTrackMetadata {
                            cloud_file_id: cloud_id,
                            cloud_path: cloud_path,
                            relative_path: t.relative_path,
                            tags: Some(tags),
                            last_modified: t.track_updated_at.timestamp_millis().to_string(),
                            last_sync: Some(Utc::now()),
                            provider: t.provider_type,
                            cloud_folder_id: t.cloud_folder_id,
                            size: t.size,
                        })
                    }
                    _ => {
                        result.tracks_skipped += 1;
                        None
                    }
                }
            })
            .collect(),
    };

    // 3. Upload to cloud - No DB lock needed
    let temp_path = temp_dir().join("syncudio_tracks.json.tmp");
    std::fs::write(&temp_path, serde_json::to_string_pretty(&metadata)?)?;
    provider
        .upload_file(&temp_path, "tracks.json", Some("/Syncudio/metadata"))
        .await?;
    std::fs::remove_file(temp_path)?;

    Ok(result)
}
