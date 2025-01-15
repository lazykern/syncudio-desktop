import { invoke } from '@tauri-apps/api/core';
import type { CloudFolder, CloudProviderType } from '../generated/typings';

export const cloudDatabase = {
  async getCloudFolder(id: string): Promise<CloudFolder | null> {
    return invoke('plugin:cloud|get_cloud_folder', { id });
  },

  async getCloudFoldersByProvider(providerType: CloudProviderType): Promise<CloudFolder[]> {
    return invoke('plugin:cloud|get_cloud_folders_by_provider', { providerType });
  },

  async getCloudFolderByLocalPath(localPath: string): Promise<CloudFolder | null> {
    return invoke('plugin:cloud|get_cloud_folder_by_local_path', { localPath });
  },

  async saveCloudFolder(folder: CloudFolder): Promise<CloudFolder> {
    return invoke('plugin:cloud|save_cloud_folder', { folder });
  },

  async updateCloudFolder(folder: CloudFolder): Promise<CloudFolder> {
    return invoke('plugin:cloud|update_cloud_folder', { folder });
  },

  async deleteCloudFolder(id: string): Promise<void> {
    return invoke('plugin:cloud|delete_cloud_folder', { id });
  }
}; 