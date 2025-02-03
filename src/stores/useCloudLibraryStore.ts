import { ask } from '@tauri-apps/plugin-dialog';
import type { CloudFolderScanResult, CloudMusicFolder } from '../generated/typings';
import { cloudDatabase } from '../lib/cloud-database';
import { cloudMetadata } from '../lib/cloud-metadata';
import { logAndNotifyError } from '../lib/utils';
import type { StateCreator } from 'zustand';
import { persist } from 'zustand/middleware';
import type { API } from '../types/syncudio';
import { createStore } from './store-helpers';
import useToastsStore from './useToastsStore';

type CloudState = API<{
  isSyncing: boolean;
  isScanning: boolean;
  scanningFolderId: string | null;
  folders: CloudMusicFolder[];
  api: {
    syncMetadata: () => Promise<void>;
    scanCloudMusicFolder: (folderId: string) => Promise<CloudFolderScanResult>;
    removeFolder: (folderId: string) => Promise<void>;
    loadFolders: () => Promise<void>;
    saveFolder: (folder: CloudMusicFolder) => Promise<void>;
  };
}>;

const useCloudStore = createCloudStore<CloudState>((set, get) => ({
  isSyncing: false,
  isScanning: false,
  scanningFolderId: null,
  folders: [],

  api: {
    syncMetadata: async () => {
      set({ isSyncing: true });
      try {
        const syncResult = await cloudMetadata.pullCloudMetadata();
        const updateResult = await cloudMetadata.pushCloudMetadata();
        
        let message = '';
        if (syncResult.is_fresh_start) {
          message = `Initial metadata sync complete. Created ${syncResult.tracks_created} tracks.`;
        } else {
          message = `Metadata sync complete. Updated ${syncResult.tracks_updated} tracks, created ${syncResult.tracks_created} tracks.`;
        }
        message += ` ${updateResult.tracks_included} tracks included in cloud metadata, ${updateResult.tracks_skipped} tracks skipped (not uploaded yet).`;

        useToastsStore.getState().api.add('success', message);
      } catch (err) {
        logAndNotifyError(err, 'Failed to sync cloud metadata');
      } finally {
        set({ isSyncing: false });
      }
    },

    scanCloudMusicFolder: async (folderId: string) => {
      const state = get();
      if (state.isScanning) {
        useToastsStore.getState().api.add('warning', 'A folder scan is already in progress');
        throw new Error('A folder scan is already in progress');
      }

      set({ isScanning: true, scanningFolderId: folderId });
      try {
        const result = await cloudDatabase.scanCloudMusicFolder(folderId);
        useToastsStore.getState().api.add(
          'success',
          `Folder scan complete:
          • ${result.cloud_tracks_found} tracks found in cloud
          • ${result.local_tracks_found} tracks found locally
          • ${result.tracks_created} new tracks created
          • ${result.tracks_updated} tracks updated
          • ${result.mappings_cleared} mappings cleared`
        );
        return result;
      } catch (err) {
        logAndNotifyError(err, 'Failed to scan folder');
        throw err;
      } finally {
        set({ isScanning: false, scanningFolderId: null });
      }
    },

    removeFolder: async (folderId: string) => {
      try {
        const confirmed = await ask('Are you sure you want to remove this cloud folder?', {
          title: 'Confirm Remove',
          kind: 'warning'
        });

        if (!confirmed) return;

        await cloudDatabase.deleteCloudFolder(folderId);
        await get().api.loadFolders();
        useToastsStore.getState().api.add('success', 'Cloud folder removed successfully');
      } catch (err) {
        logAndNotifyError(err, 'Failed to remove cloud folder');
      }
    },

    loadFolders: async () => {
      try {
        const folders = await cloudDatabase.getCloudFolders();
        set({ folders });
      } catch (err) {
        logAndNotifyError(err, 'Failed to load cloud folders');
      }
    },

    saveFolder: async (folder: CloudMusicFolder) => {
      try {
        await cloudDatabase.saveCloudFolder(folder);
        await get().api.loadFolders();
        useToastsStore.getState().api.add('success', 'Cloud folder added successfully');
      } catch (err) {
        logAndNotifyError(err, 'Failed to add cloud folder');
      }
    },
  },
}));

export default useCloudStore;

export function useCloudAPI() {
  return useCloudStore((state) => state.api);
}

function createCloudStore<T extends CloudState>(store: StateCreator<T>) {
  return createStore(
    persist(store, {
      name: 'syncudio-cloud',
      merge(persistedState, currentState) {
        const mergedState = {
          ...currentState,
          // API should never be persisted
          api: currentState.api,
        };

        if (persistedState != null && typeof persistedState === 'object') {
          const state = persistedState as { isSyncing?: boolean; isScanning?: boolean; scanningFolderId?: string | null };
          if ('isSyncing' in state) {
            state.isSyncing = false;
          }
          if ('isScanning' in state) {
            state.isScanning = false;
            state.scanningFolderId = null;
          }
        }

        return mergedState;
      },
    }),
  );
}
