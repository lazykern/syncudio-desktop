import { invoke } from '@tauri-apps/api/core';
import type { CloudMusicFolder, CloudProviderType, UnifiedTrack } from '../generated/typings';

export const cloudDatabase = {
  async getCloudFolders(): Promise<CloudMusicFolder[]> {
    return invoke('plugin:cloud|get_cloud_music_folders');
  },

  async getCloudFoldersByProvider(providerType: CloudProviderType): Promise<CloudMusicFolder[]> {
    return invoke('plugin:cloud|get_cloud_music_folders_by_provider', { providerType });
  },

  async getCloudFolderByLocalPath(localPath: string): Promise<CloudMusicFolder | null> {
    return invoke('plugin:cloud|get_cloud_folder_by_local_path', { localPath });
  },

  async saveCloudFolder(folder: CloudMusicFolder): Promise<CloudMusicFolder> {
    return invoke('plugin:cloud|save_cloud_folder', { folder });
  },

  async updateCloudFolder(folder: CloudMusicFolder): Promise<CloudMusicFolder> {
    return invoke('plugin:cloud|update_cloud_folder', { folder });
  },

  async deleteCloudFolder(id: string): Promise<void> {
    return invoke('plugin:cloud|delete_cloud_folder', { id });
  },

  /**
   * Discovers and syncs tracks in a cloud folder.
   * Should be called:
   * 1. When a new cloud folder is mapped
   * 2. During manual sync operations
   * 3. During periodic background scans
   * 4. When file system changes are detected
   */
  async discoverCloudFolderTracks(folderId: string): Promise<void> {
    return invoke('plugin:cloud|discover_cloud_folder_tracks', { folderId });
  },

  /**
   * Syncs cloud tracks metadata across devices.
   * Should be called:
   * 1. After discovering tracks in a cloud folder
   * 2. During manual sync operations
   * 3. Periodically to ensure metadata consistency
   * 4. When local track changes are detected
   */
  async syncCloudTracksMetadata(providerType: CloudProviderType): Promise<void> {
    return invoke('plugin:cloud|sync_cloud_tracks_metadata', { providerType });
  },

  /**
   * Cleans up missing local tracks and their related cloud mappings.
   * Should be called:
   * 1. Before library refresh operations
   * 2. After detecting file system changes
   * 3. Before cloud sync operations
   */
  async cleanupMissingTracks(): Promise<{
    removed_tracks: number;
    removed_cloud_mappings: number;
    removed_cloud_tracks: number;
  }> {
    return invoke('plugin:cloud|cleanup_missing_local_tracks');
  },

  /**
   * Gets all unified tracks from both local and cloud sources
   */
  async getUnifiedTracks(): Promise<UnifiedTrack[]> {
    return invoke('plugin:cloud|get_unified_tracks');
  },

  /**
   * Gets unified tracks by their local or cloud IDs
   */
  async getUnifiedTracksByIds(ids: string[]): Promise<UnifiedTrack[]> {
    return invoke('plugin:cloud|get_unified_tracks_by_ids', { ids });
  },

  /**
   * Gets unified tracks by their location type ('local', 'cloud', or 'both')
   */
  async getUnifiedTracksByLocation(locationType: 'local' | 'cloud' | 'both'): Promise<UnifiedTrack[]> {
    return invoke('plugin:cloud|get_unified_tracks_by_location', { locationType });
  },

  /**
   * Gets unified tracks by cloud folder ID
   */
  async getUnifiedTracksByFolder(folderId: string): Promise<UnifiedTrack[]> {
    return invoke('plugin:cloud|get_unified_tracks_by_folder', { folderId });
  },

  /**
   * Gets unified tracks by cloud provider type
   */
  async getUnifiedTracksByProvider(providerType: CloudProviderType): Promise<UnifiedTrack[]> {
    return invoke('plugin:cloud|get_unified_tracks_by_provider', { providerType });
  },

  /**
   * Gets a single unified track by its local or cloud ID
   */
  async getUnifiedTrack(id: string): Promise<UnifiedTrack | null> {
    return invoke('plugin:cloud|get_unified_track', { id });
  }
};
