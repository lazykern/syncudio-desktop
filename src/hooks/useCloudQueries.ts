import { useQuery, useQueries } from '@tanstack/react-query';
import { cloudSync } from '../lib/cloud-sync';
import type { CloudFolder, CloudFolderSyncDetailsDTO, QueueItemDTO, QueueStatsDTO } from '../generated/typings';

// Query keys
export const cloudKeys = {
  folders: ['cloud', 'folders'] as const,
  folderDetails: (folderId: string | null) => ['cloud', 'folder', 'details', folderId] as const,
  queueItems: (folderId: string | undefined) => ['cloud', 'queue', 'items', folderId] as const,
  queueStats: (folderId: string | undefined) => ['cloud', 'queue', 'stats', folderId] as const,
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
    queryFn: () => cloudSync.getCloudFolders(),
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