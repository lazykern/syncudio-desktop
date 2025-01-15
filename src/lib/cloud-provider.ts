import { invoke } from '@tauri-apps/api/core';
import type { CloudFile } from '../generated/typings';

export const CLOUD_PROVIDER_DROPBOX = 'dropbox';
export const CLOUD_PROVIDER_GDRIVE = 'gdrive';

export type CloudProvider = typeof CLOUD_PROVIDER_DROPBOX | typeof CLOUD_PROVIDER_GDRIVE;

export const cloud = {
  // Dropbox-specific auth methods
  async dropboxStartAuthorization(): Promise<string> {
    return invoke('plugin:cloud|dropbox_start_auth');
  },

  async dropboxCompleteAuthorization(authCode: string): Promise<void> {
    return invoke('plugin:cloud|dropbox_complete_auth', { authCode });
  },

  async dropboxUnauthorize(): Promise<void> {
    return invoke('plugin:cloud|dropbox_unauthorize');
  },

  async dropboxIsAuthorized(): Promise<boolean> {
    return invoke('plugin:cloud|dropbox_is_authorized');
  },

  // Generic cloud operations
  async listFiles(providerType: CloudProvider, folderId: string, recursive = false): Promise<CloudFile[]> {
    return invoke('plugin:cloud|cloud_list_files', { providerType, folderId, recursive });
  },

  async listRootFiles(providerType: CloudProvider, recursive = false): Promise<CloudFile[]> {
    return invoke('plugin:cloud|cloud_list_root_files', { providerType, recursive });
  },

  async createFolder(providerType: CloudProvider, name: string, parentId: string | null): Promise<CloudFile> {
    return invoke('plugin:cloud|cloud_create_folder', { providerType, name, parentId });
  },

  async uploadFile(providerType: CloudProvider, localPath: string, name: string, parentId: string | null): Promise<CloudFile> {
    return invoke('plugin:cloud|cloud_upload_file', { providerType, absLocalPath: localPath, name, parentId });
  },

  async downloadFile(providerType: CloudProvider, fileId: string, localPath: string): Promise<void> {
    return invoke('plugin:cloud|cloud_download_file', { providerType, fileId, absLocalPath: localPath });
  },

  async deleteFile(providerType: CloudProvider, fileId: string): Promise<void> {
    return invoke('plugin:cloud|cloud_delete_file', { providerType, fileId });
  }
};
