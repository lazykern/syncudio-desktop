import { useQuery, useQueries, useMutation, useQueryClient } from '@tanstack/react-query';
import { cloudSync } from '../lib/cloud-sync';
import type { CloudFolder, CloudFolderSyncDetailsDTO, QueueItemDTO, QueueStatsDTO, TrackSyncStatusDTO } from '../generated/typings';
import { cloudDatabase } from '../lib/cloud-database';

// Query keys
export const cloudKeys = {
  folders: ['cloud', 'folders'] as const,
  folderDetails: (folderId: string | null) => ['cloud', 'folder', 'details', folderId] as const,
  queueItems: (folderId: string | undefined) => ['cloud', 'queue', 'items', folderId] as const,
  queueStats: (folderId: string | undefined) => ['cloud', 'queue', 'stats', folderId] as const,
  trackSyncStatus: (trackId: string) => ['cloud', 'track', 'sync', trackId] as const,
};

// Types
export type FolderWithDetails = CloudFolder & {
  details?: CloudFolderSyncDetailsDTO;
};

// Hooks
export function useCloudFolders() {
  // First get all folders
  const foldersQuery = useQuery<CloudFolder[]>({
    queryKey: cloudKeys.folders,
    queryFn: () => cloudDatabase.getCloudFolders(),
  });

  // Then get details for each folder
  const detailsQueries = useQueries({
    queries: (foldersQuery.data || []).map((folder) => ({
      queryKey: cloudKeys.folderDetails(folder.id),
      queryFn: () => cloudSync.getCloudFolderSyncDetails(folder.id),
      enabled: !!folder.id,
    })),
  });

  // Combine folders with their details
  const foldersWithDetails: FolderWithDetails[] = (foldersQuery.data || []).map((folder, index) => ({
    ...folder,
    details: detailsQueries[index].data,
  }));

  return {
    ...foldersQuery,
    data: foldersWithDetails,
  };
}

export function useCloudFolderDetails(folderId: string | null) {
  return useQuery<CloudFolderSyncDetailsDTO>({
    queryKey: cloudKeys.folderDetails(folderId),
    queryFn: () => (folderId ? cloudSync.getCloudFolderSyncDetails(folderId) : Promise.reject('No folder selected')),
    enabled: !!folderId,
  });
}

export function useQueueItems(folderId: string | undefined) {
  return useQuery<QueueItemDTO[]>({
    queryKey: cloudKeys.queueItems(folderId),
    queryFn: () => cloudSync.getQueueItems(folderId),
    refetchInterval: 5000, // Poll every 5 seconds
  });
}

export function useQueueStats(folderId: string | undefined) {
  return useQuery<QueueStatsDTO>({
    queryKey: cloudKeys.queueStats(folderId),
    queryFn: () => cloudSync.getQueueStats(folderId),
    refetchInterval: 5000, // Poll every 5 seconds
  });
}

export function useTrackSyncStatus(trackId: string) {
  return useQuery<TrackSyncStatusDTO>({
    queryKey: cloudKeys.trackSyncStatus(trackId),
    queryFn: () => cloudSync.getTrackSyncStatus(trackId),
    enabled: !!trackId,
  });
}

export function useSyncMutations(folderId?: string) {
  const queryClient = useQueryClient();

  // Keys that need to be invalidated after sync operations
  const invalidateKeys = [
    ['cloudFolderDetails', folderId],
    ['queueItems', folderId],
    ['queueStats', folderId],
  ];

  const invalidateQueries = async () => {
    await Promise.all(
      invalidateKeys.map(key => queryClient.invalidateQueries({ queryKey: key }))
    );
  };

  const uploadMutation = useMutation({
    mutationFn: (trackIds: string[]) => cloudSync.addToUploadQueue(trackIds),
    onSuccess: () => invalidateQueries(),
  });

  const downloadMutation = useMutation({
    mutationFn: (trackIds: string[]) => cloudSync.addToDownloadQueue(trackIds),
    onSuccess: () => invalidateQueries(),
  });

  return {
    uploadMutation,
    downloadMutation,
    isLoading: uploadMutation.isPending || downloadMutation.isPending,
  };
} 