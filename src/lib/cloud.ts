import { invoke } from '@tauri-apps/api/core';

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
};
