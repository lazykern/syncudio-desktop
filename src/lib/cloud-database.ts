import { invoke } from '@tauri-apps/api/core';
import type { CloudMusicFolder, CloudProviderType } from '../generated/typings';

export const cloudDatabase = {
  async getCloudFolders(): Promise<CloudMusicFolder[]> {
    return invoke('plugin:cloud|get_cloud_folders');
  },

  async getCloudFoldersByProvider(providerType: CloudProviderType): Promise<CloudMusicFolder[]> {
    return invoke('plugin:cloud|get_cloud_folders_by_provider', { providerType });
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
  }
}; 