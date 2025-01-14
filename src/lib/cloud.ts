import { invoke } from '@tauri-apps/api/core';

export interface CloudFile {
  id: string;
  name: string;
  parent_id: string | null;
  is_folder: boolean;
}

export const cloud = {
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

  async dropboxListFiles(folderId: string): Promise<CloudFile[]> {
    return invoke('plugin:cloud|dropbox_list_files', { folderId });
  },

  async dropboxListFilesRecursive(folderId: string): Promise<CloudFile[]> {
    return invoke('plugin:cloud|dropbox_list_files_recursive', { folderId });
  },

  async dropboxCreateFolder(name: string, parentId: string | null): Promise<CloudFile> {
    return invoke('plugin:cloud|dropbox_create_folder', { name, parentId });
  },

  async dropboxUploadFile(localPath: string, name: string, parentId: string | null): Promise<CloudFile> {
    return invoke('plugin:cloud|dropbox_upload_file', { absLocalPath: localPath, name, parentId });
  },

  async dropboxDownloadFile(fileId: string, localPath: string): Promise<void> {
    return invoke('plugin:cloud|dropbox_download_file', { fileId, absLocalPath: localPath });
  },

  async dropboxDeleteFile(fileId: string): Promise<void> {
    return invoke('plugin:cloud|dropbox_delete_file', { fileId });
  }
};
