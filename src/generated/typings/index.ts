// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

/**
 * Result of a cleanup operation
 */
export type CleanupResult = { removed_tracks: number, removed_cloud_mappings: number, removed_cloud_tracks: number, };

export type CloudFile = { id: string, name: string, size: bigint, is_folder: boolean, modified_at: string, mime_type: string | null, hash: FileHash | null, display_path: string | null, relative_path: string, };

export type CloudFolderScanResult = { 
/**
 * Number of tracks found in cloud storage
 */
cloud_tracks_found: number, 
/**
 * Number of local tracks found in the folder
 */
local_tracks_found: number, 
/**
 * Number of tracks that were newly created in cloud_tracks table
 */
tracks_created: number, 
/**
 * Number of tracks that were updated with new information
 */
tracks_updated: number, 
/**
 * Number of track mappings that were cleared (cloud_file_id set to None)
 */
mappings_cleared: number, };

/**
 * Represents detailed sync information for a cloud folder
 */
export type CloudFolderSyncDetailsDTO = { id: string, cloud_folder_path: string, local_folder_path: string, sync_status: FolderSyncStatus, pending_sync_count: number, tracks: Array<CloudTrackDTO>, };

/**
 * Collection of track metadata for cloud storage
 */
export type CloudMetadataCollection = { tracks: Array<CloudTrackMetadata>, last_updated: string, version: string, };

/**
 * Result of a metadata sync operation
 */
export type CloudMetadataSyncResult = { tracks_updated: number, tracks_created: number, metadata_version: string, is_fresh_start: boolean, };

/**
 * Result of a metadata update operation
 */
export type CloudMetadataUpdateResult = { tracks_included: number, tracks_skipped: number, metadata_version: string, };

export type CloudMusicFolder = { id: string, provider_type: string, cloud_folder_id: string, cloud_folder_path: string, local_folder_path: string, };

export type CloudProviderType = "dropbox" | "gdrive";

export type CloudTrack = { id: string, blake3_hash: string | null, file_name: string, updated_at: string, tags: CloudTrackTag | null, };

/**
 * Represents a track with its current sync and integrity status
 */
export type CloudTrackDTO = { id: string, cloud_music_folder_id: string, cloud_map_id: string, file_name: string, relative_path: string, location_state: TrackLocationState, sync_operation: SyncOperationType | null, sync_status: SyncStatus | null, updated_at: string, tags: CloudTrackTag | null, };

/**
 * Comprehensive DTO that combines CloudTrack, CloudTrackMap, and CloudMusicFolder
 * Used for efficient lookups and metadata operations
 */
export type CloudTrackFullDTO = { track_id: string, blake3_hash: string | null, file_name: string, track_updated_at: string, tags: CloudTrackTag | null, map_id: string, cloud_file_id: string | null, relative_path: string, folder_id: string, provider_type: string, cloud_folder_id: string, cloud_folder_path: string, local_folder_path: string, };

export type CloudTrackMap = { id: string, cloud_track_id: string, cloud_music_folder_id: string, cloud_file_id: string | null, relative_path: string, };

/**
 * Represents track metadata stored in cloud storage
 */
export type CloudTrackMetadata = { blake3_hash: string, cloud_file_id: string, cloud_path: string, relative_path: string, tags: CloudTrackTag | null, last_modified: string, last_sync: string, provider: string, cloud_folder_id: string, };

export type CloudTrackTag = { title: string, album: string, artists: Array<string>, genres: Array<string>, year: number | null, duration: number, track_no: number | null, track_of: number | null, disk_no: number | null, disk_of: number | null, };

export type Config = { theme: string, audio_volume: number, audio_playback_rate: number | null, audio_output_device: string, audio_muted: boolean, audio_shuffle: boolean, audio_repeat: Repeat, default_view: DefaultView, library_sort_by: SortBy, library_sort_order: SortOrder, library_folders: Array<string>, library_autorefresh: boolean, sleepblocker: boolean, auto_update_checker: boolean, minimize_to_tray: boolean, notifications: boolean, track_view_density: string, sync_worker_enabled: boolean, sync_concurrent_uploads: number, sync_concurrent_downloads: number, sync_retry_limit: number, sync_retry_delay_seconds: number, lastfm_enabled: boolean, };

export type DefaultView = "Library" | "Playlists";

export type DownloadQueueItem = { id: string, priority: number, cloud_map_id: string, provider_type: string, status: string, error_message: string | null, created_at: string, updated_at: string, attempts: number, };

export type FileHash = { "Sha1": string } | { "Sha256": string } | { "ContentHash": string };

/**
 * Represents the sync status of a cloud folder
 */
export type FolderSyncStatus = "synced" | "syncing" | "needs_attention" | "empty";

export type IPCEvent = { "Unknown": string } | "PlaybackPlay" | "PlaybackPause" | "PlaybackStop" | "PlaybackPlayPause" | "PlaybackPrevious" | "PlaybackNext" | "PlaybackStart" | "LibraryScanProgress" | "GoToLibrary" | "GoToPlaylists" | "GoToSettings" | "JumpToPlayingTrack";

/** ----------------------------------------------------------------------------
 * Playlist
 * represent a playlist, that has a name and a list of tracks
 * -------------------------------------------------------------------------- */
export type Playlist = { id: string, name: string, tracks: Array<string>, import_path: string | null, };

/**
 * Represents a sync queue item
 */
export type QueueItemDTO = { id: string, cloud_track_id: string, file_name: string, operation: SyncOperationType, status: SyncStatus, created_at: string, updated_at: string, provider_type: string, };

/**
 * Represents queue statistics
 */
export type QueueStatsDTO = { pending_count: number, in_progress_count: number, completed_count: number, failed_count: number, };

export type Repeat = "All" | "One" | "None";

/**
 * Scan progress information
 */
export type ScanProgress = { current: number, total: number, };

/**
 * Result of a library scan operation
 */
export type ScanResult = { track_count: number, track_failures: number, playlist_count: number, playlist_failures: number, };

export type SortBy = "Artist" | "Album" | "Title" | "Duration" | "Genre";

export type SortOrder = "Asc" | "Dsc";

/**
 * Represents a sync history entry
 */
export type SyncHistoryEntry = { timestamp: string, operation: SyncOperationType, old_hash: string | null, new_hash: string | null, status: SyncStatus, };

/**
 * Represents operation type for sync operations
 */
export type SyncOperationType = "upload" | "download";

export type SyncQueueStatus = "pending" | "in_progress" | "completed" | "failed" | "cancelled";

/**
 * Represents the status of a sync operation
 */
export type SyncStatus = "pending" | "in_progress" | "completed" | { "failed": { error: string, attempts: number, } };

/**
 * Track
 * represent a single track, id and path should be unique
 */
export type Track = { id: string, blake3_hash: string | null, path: string, title: string, album: string, artists: Array<string>, genres: Array<string>, year: number | null, duration: number, track_no: number | null, track_of: number | null, disk_no: number | null, disk_of: number | null, };

export type TrackDownloadedPayload = { track_id: string, location_type: string, local_track_id: string, cloud_track_id: string, sync_folder_id: string, relative_path: string, };

/**
 * Represents the location state of a track by checking both local and cloud existence by blake3_hash, cloud_file_id and relative_path (should be in local storage and cloud storage)
 */
export type TrackLocationState = "complete" | "local_only" | "cloud_only" | "out_of_sync" | "missing" | "not_mapped";

/**
 * Represents detailed sync information for a track
 */
export type TrackSyncStatusDTO = { location_state: TrackLocationState, sync_operation: SyncOperationType | null, sync_status: SyncStatus | null, updated_at: string, };

export type UnifiedTrack = { local_track_id: string | null, cloud_track_id: string | null, cloud_map_id: string | null, cloud_folder_id: string | null, blake3_hash: string | null, local_path: string | null, cloud_relative_path: string | null, cloud_folder_path: string | null, cloud_local_folder_path: string | null, cloud_provider_type: string | null, cloud_file_id: string | null, title: string, album: string, artists: Array<string> | null, genres: Array<string> | null, year: number | null, duration: number, track_no: number | null, track_of: number | null, disk_no: number | null, disk_of: number | null, location_type: string, cloud_updated_at: string | null, };

export type UploadQueueItem = { id: string, priority: number, cloud_map_id: string, provider_type: string, status: string, error_message: string | null, created_at: string, updated_at: string, attempts: number, };
