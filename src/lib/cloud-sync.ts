import { invoke } from '@tauri-apps/api/core';
import type {
  CloudTrackDTO,
  CloudFolder,
  CloudFolderSyncDetailsDTO,
  QueueItemDTO,
  QueueStatsDTO,
  TrackSyncDetailsDTO,
  SyncHistoryEntry,
  TrackLocationState,
  SyncOperationType,
  SyncStatus,
  FolderSyncStatus,
} from '../generated/typings';

// Mock data
const mockFolders: CloudFolder[] = [
  {
    id: '1',
    provider_type: 'dropbox',
    cloud_folder_id: 'dbx_1',
    cloud_folder_path: '/Music',
    local_folder_path: '/home/user/Music',
  },
  {
    id: '2',
    provider_type: 'dropbox',
    cloud_folder_id: 'dbx_2',
    cloud_folder_path: '/Playlists',
    local_folder_path: '/home/user/Playlists',
  },
  {
    id: '3',
    provider_type: 'dropbox',
    cloud_folder_id: 'dbx_3',
    cloud_folder_path: '/Podcasts',
    local_folder_path: '/home/user/Podcasts',
  },
];

const mockFolderDetails: CloudFolderSyncDetailsDTO[] = [
  {
    id: '1',
    cloud_folder_path: '/Music',
    local_folder_path: '/home/user/Music',
    sync_status: 'syncing',
    pending_sync_count: 1,
    tracks: [
      {
        id: '1',
        file_name: 'song1.mp3',
        relative_path: 'song1.mp3',
        location_state: 'complete',
        sync_operation: null,
        sync_status: null,
        updated_at: new Date(Date.now()).toISOString(),
        tags: {
          title: 'Song 1',
          album: 'Album 1',
          artists: ['Artist 1'],
          genres: ['Rock'],
          year: 2024,
          duration: 180,
          track_no: 1,
          track_of: 12,
          disk_no: 1,
          disk_of: 1,
        },
      },
      {
        id: '2',
        file_name: 'local_only.mp3',
        relative_path: 'local_only.mp3',
        location_state: 'local_only',
        sync_operation: 'upload',
        sync_status: 'in_progress',
        updated_at: new Date(Date.now() - 3600000).toISOString(),
        tags: {
          title: 'Local Only Track',
          album: 'Local Album',
          artists: ['Local Artist'],
          genres: ['Jazz'],
          year: 2024,
          duration: 240,
          track_no: 1,
          track_of: 10,
          disk_no: 1,
          disk_of: 1,
        },
      },
    ],
  },
  {
    id: '2',
    cloud_folder_path: '/Playlists',
    local_folder_path: '/home/user/Playlists',
    sync_status: 'needs_attention',
    pending_sync_count: 2,
    tracks: [
      {
        id: '3',
        file_name: 'out_of_sync.mp3',
        relative_path: 'out_of_sync.mp3',
        location_state: 'out_of_sync',
        sync_operation: null,
        sync_status: null,
        updated_at: new Date(Date.now() - 7200000).toISOString(),
        tags: {
          title: 'Out of Sync Track',
          album: 'Some Album',
          artists: ['Some Artist'],
          genres: ['Pop'],
          year: 2023,
          duration: 200,
          track_no: 3,
          track_of: 12,
          disk_no: 1,
          disk_of: 1,
        },
      },
      {
        id: '4',
        file_name: 'cloud_only.mp3',
        relative_path: 'cloud_only.mp3',
        location_state: 'cloud_only',
        sync_operation: 'download',
        sync_status: 'pending',
        updated_at: new Date(Date.now() - 10800000).toISOString(),
        tags: {
          title: 'Cloud Only Track',
          album: 'Cloud Album',
          artists: ['Cloud Artist'],
          genres: ['Electronic'],
          year: 2024,
          duration: 320,
          track_no: 1,
          track_of: 1,
          disk_no: 1,
          disk_of: 1,
        },
      },
    ],
  },
  {
    id: '3',
    cloud_folder_path: '/Podcasts',
    local_folder_path: '/home/user/Podcasts',
    sync_status: 'empty',
    pending_sync_count: 0,
    tracks: [],
  },
];

const mockQueueItems: QueueItemDTO[] = [
  {
    id: '1',
    cloud_track_id: '2',
    file_name: 'local_only.mp3',
    operation: 'upload',
    status: 'in_progress',
    created_at: new Date(Date.now() - 60000).toISOString(),
    updated_at: new Date(Date.now()).toISOString(),
    provider_type: 'dropbox',
  },
  {
    id: '2',
    cloud_track_id: '4',
    file_name: 'cloud_only.mp3',
    operation: 'download',
    status: 'pending',
    created_at: new Date(Date.now() - 30000).toISOString(),
    updated_at: new Date(Date.now()).toISOString(),
    provider_type: 'dropbox',
  },
];

const mockQueueStats: QueueStatsDTO = {
  pending_count: 1,
  in_progress_count: 1,
  completed_count: 5,
  failed_count: 0,
};

const mockSyncHistory: SyncHistoryEntry[] = [
  {
    timestamp: new Date(Date.now() - 3600000).toISOString(),
    operation: 'upload',
    old_hash: 'abc123',
    new_hash: 'def456',
    status: 'completed',
  },
  {
    timestamp: new Date(Date.now() - 7200000).toISOString(),
    operation: 'download',
    old_hash: null,
    new_hash: 'ghi789',
    status: 'completed',
  },
  {
    timestamp: new Date(Date.now() - 10800000).toISOString(),
    operation: 'upload',
    old_hash: 'xyz987',
    new_hash: null,
    status: { failed: { error: 'Network error', attempts: 3 } },
  },
];

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
    return mockQueueItems;
  },

  /**
   * Get queue statistics
   */
  async getQueueStats(folderId?: string): Promise<QueueStatsDTO> {
    return mockQueueStats;
  },

  /**
   * Force sync a specific folder
   */
  async forceSyncFolder(folderId: string): Promise<void> {
    console.log('Force syncing folder:', folderId);
  },

  /**
   * Pause or resume sync operations
   */
  async setSyncPaused(paused: boolean): Promise<void> {
    console.log('Setting sync paused:', paused);
  },

  /**
   * Retry failed sync items for a folder or all folders
   */
  async retryFailedItems(folderId?: string): Promise<void> {
    console.log('Retrying failed items for folder:', folderId);
  },

  /**
   * Get detailed sync information for a track
   */
  async getTrackSyncDetails(trackId: string): Promise<TrackSyncDetailsDTO> {
    const track = mockFolderDetails
      .flatMap(f => f.tracks)
      .find(t => t.id === trackId);
      
    if (!track) {
      throw new Error('Track not found');
    }

    return {
      track,
      sync_history: mockSyncHistory,
      current_operation: mockQueueItems.find(q => q.cloud_track_id === trackId) || null,
    };
  },
};
