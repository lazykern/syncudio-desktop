import { invoke } from '@tauri-apps/api/core';
import type { CloudMetadataSyncResult, CloudMetadataUpdateResult } from '../generated/typings';

export const cloudMetadata = {
  /**
   * Sync metadata from cloud to local database
   * @returns Details about the sync operation
   */
  async syncCloudMetadata(): Promise<CloudMetadataSyncResult> {
    return invoke('plugin:cloud|sync_cloud_metadata');
  },

  /**
   * Update cloud metadata with current database state
   */
  async updateCloudMetadata(): Promise<CloudMetadataUpdateResult> {
    return invoke('plugin:cloud|update_cloud_metadata');
  },
}; 