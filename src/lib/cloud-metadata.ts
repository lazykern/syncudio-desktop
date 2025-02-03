import { invoke } from '@tauri-apps/api/core';
import type { CloudMetadataSyncResult, CloudMetadataUpdateResult } from '../generated/typings';

export const cloudMetadata = {
  /**
   * Pull metadata from cloud to local database
   * @returns Details about the pull operation
   */
  async pullCloudMetadata(): Promise<CloudMetadataSyncResult> {
    return invoke('plugin:cloud|pull_cloud_metadata');
  },

  /**
   * Push local metadata to cloud
   * @returns Details about the push operation
   */
  async pushCloudMetadata(): Promise<CloudMetadataUpdateResult> {
    return invoke('plugin:cloud|push_cloud_metadata');
  },
};
