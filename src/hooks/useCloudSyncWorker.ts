import { useEffect, useRef } from 'react';
import { QueueItemDTO } from '../generated/typings';
import { cloudSync } from '../lib/cloud-sync';
import { signal } from '@preact/signals-react';
import { useQueryClient } from '@tanstack/react-query';
import { cloudKeys } from './useCloudQueries';

interface SyncWorkerState {
  isRunning: boolean;
  isPaused: boolean;
  activeUploads: Map<string, QueueItemDTO>;
  activeDownloads: Map<string, QueueItemDTO>;
}

// Create signals for worker state
const workerState = signal<SyncWorkerState>({
  isRunning: false,
  isPaused: false,
  activeUploads: new Map(),
  activeDownloads: new Map(),
});

export function useCloudSyncWorker() {
  const queryClient = useQueryClient();
  const isInitialized = useRef(false);

  const invalidateQueries = async () => {
    // Invalidate all folder details as sync status might have changed
    await queryClient.invalidateQueries({ 
      queryKey: ['cloud', 'folder', 'details']
    });
    
    // Invalidate queue items and stats
    await queryClient.invalidateQueries({ 
      queryKey: ['cloud', 'queue']
    });

    // Invalidate all track sync statuses
    await queryClient.invalidateQueries({
      queryKey: ['cloud', 'track', 'sync']
    });
  };

  // Handle stale "in_progress" items on startup
  useEffect(() => {
    const resetStaleItems = async () => {
      if (isInitialized.current) return;
      
      try {
        await cloudSync.resetInProgressItems();
        // Start the worker after resetting stale items
        workerState.value = { ...workerState.value, isRunning: true };
        await invalidateQueries();
        isInitialized.current = true;
      } catch (error) {
        console.error('Failed to reset stale items:', error);
      }
    };
    resetStaleItems();
  }, []);

  // Main worker loop
  useEffect(() => {
    if (!workerState.value.isRunning || workerState.value.isPaused) return;

    const processQueues = async () => {
      // Process uploads (max 3 concurrent)
      while (workerState.value.activeUploads.size < 3) {
        try {
          const nextItem = await cloudSync.getNextUploadItem();
          if (!nextItem) break;
          processUpload(nextItem);
        } catch (error) {
          console.error('Failed to get next upload item:', error);
          break;
        }
        await new Promise(resolve => setTimeout(resolve, 1000));
      }

      // Process downloads (max 3 concurrent)
      while (workerState.value.activeDownloads.size < 3) {
        try {
          const nextItem = await cloudSync.getNextDownloadItem();
          if (!nextItem) break;
          processDownload(nextItem);
        } catch (error) {
          console.error('Failed to get next download item:', error);
          break;
        }
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    };

    const interval = setInterval(processQueues, 1000);
    return () => clearInterval(interval);
  }, []);

  const processUpload = async (item: QueueItemDTO) => {
    try {
      // Add to active uploads
      const newUploads = new Map(workerState.value.activeUploads);
      newUploads.set(item.id, item);
      workerState.value = { ...workerState.value, activeUploads: newUploads };

      await cloudSync.startUpload(item.id);
      await invalidateQueries();

      // Remove from active uploads
      const updatedUploads = new Map(workerState.value.activeUploads);
      updatedUploads.delete(item.id);
      workerState.value = { ...workerState.value, activeUploads: updatedUploads };
      await invalidateQueries();
    } catch (error) {
      console.error(`Upload failed for item ${item.id}:`, error);
      await cloudSync.failUpload(item.id, error instanceof Error ? error.message : String(error));

      // Remove from active uploads
      const updatedUploads = new Map(workerState.value.activeUploads);
      updatedUploads.delete(item.id);
      workerState.value = { ...workerState.value, activeUploads: updatedUploads };
      await invalidateQueries();
    }
  };

  const processDownload = async (item: QueueItemDTO) => {
    try {
      // Add to active downloads
      const newDownloads = new Map(workerState.value.activeDownloads);
      newDownloads.set(item.id, item);
      workerState.value = { ...workerState.value, activeDownloads: newDownloads };

      await cloudSync.startDownload(item.id);
      await invalidateQueries();

      // Remove from active downloads
      const updatedDownloads = new Map(workerState.value.activeDownloads);
      updatedDownloads.delete(item.id);
      workerState.value = { ...workerState.value, activeDownloads: updatedDownloads };
      await invalidateQueries();
    } catch (error) {
      console.error(`Download failed for item ${item.id}:`, error);
      await cloudSync.failDownload(item.id, error instanceof Error ? error.message : String(error));

      // Remove from active downloads
      const updatedDownloads = new Map(workerState.value.activeDownloads);
      updatedDownloads.delete(item.id);
      workerState.value = { ...workerState.value, activeDownloads: updatedDownloads };
      await invalidateQueries();
    }
  };

  return {
    startWorker: () => {
      workerState.value = { ...workerState.value, isRunning: true };
    },
    stopWorker: () => {
      workerState.value = { ...workerState.value, isRunning: false };
    },
    pauseWorker: () => {
      workerState.value = { ...workerState.value, isPaused: true };
    },
    resumeWorker: () => {
      workerState.value = { ...workerState.value, isPaused: false };
    },
    state: workerState.value
  };
}
