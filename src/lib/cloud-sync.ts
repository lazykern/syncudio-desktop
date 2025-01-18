import { invoke } from '@tauri-apps/api/core';
import type {
  CloudTrackDTO,
  CloudFolder,
  CloudFolderSyncDetailsDTO,
  QueueItemDTO,
  QueueStatsDTO,
  TrackSyncStatusDTO,
  TrackLocationState,
  SyncOperationType,
  SyncStatus,
  FolderSyncStatus,
} from '../generated/typings';

export const cloudSync = {
  /**
   * Get detailed sync information for a cloud folder
   */
  async getCloudFolderSyncDetails(folderId: string): Promise<CloudFolderSyncDetailsDTO> {
    return invoke('plugin:cloud|get_cloud_folder_sync_details', { folderId });
  },

  /**
   * Get active queue items
   */
  async getQueueItems(folderId?: string): Promise<QueueItemDTO[]> {
    return invoke('plugin:cloud|get_queue_items', { folderId });
  },

  /**
   * Get queue statistics
   */
  async getQueueStats(folderId?: string): Promise<QueueStatsDTO> {
    return invoke('plugin:cloud|get_queue_stats', { folderId });
  },

  /**
   * Add tracks to the upload queue
   */
  async addToUploadQueue(trackIds: string[]): Promise<void> {
    return invoke('plugin:cloud|add_to_upload_queue', { trackIds });
  },

  /**
   * Add tracks to the download queue
   */
  async addToDownloadQueue(trackIds: string[]): Promise<void> {
    return invoke('plugin:cloud|add_to_download_queue', { trackIds });
  },

  /**
   * Pause or resume sync operations
   */
  async setSyncPaused(paused: boolean): Promise<void> {
    return invoke('plugin:cloud|set_sync_paused', { paused });
  },

  /**
   * Retry failed sync items for a folder or all folders
   */
  async retryFailedItems(folderId?: string): Promise<void> {
    return invoke('plugin:cloud|retry_failed_items', { folderId });
  },

  /**
   * Get sync status for a track
   */
  async getTrackSyncStatus(trackId: string): Promise<TrackSyncStatusDTO> {
    return invoke('plugin:cloud|get_track_sync_status', { trackId });
  },
};
