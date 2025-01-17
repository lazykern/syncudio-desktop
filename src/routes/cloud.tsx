import { useState } from 'react';
import type { CloudTrack, CloudFolder } from '../generated/typings';
import styles from './cloud.module.css';

// Enums (these should match the Rust enums)
enum TrackIntegrityStatus {
  Complete = 'complete',
  LocalOnly = 'local_only',
  CloudOnly = 'cloud_only',
  OutOfSync = 'out_of_sync',
  Missing = 'missing',
  NotMapped = 'not_mapped',
}

enum SyncOperationType {
  Upload = 'upload',
  Download = 'download',
}

enum SyncStatus {
  Pending = 'pending',
  InProgress = 'in_progress',
  Completed = 'completed',
  Failed = 'failed',
}

enum FolderSyncStatus {
  Synced = 'synced',
  Syncing = 'syncing',
  NeedsAttention = 'needs_attention',
  Empty = 'empty',
}

// Types that match our DTOs
type CloudTrackDTO = {
  id: string;
  file_name: string;
  relative_path: string;
  integrity_status: TrackIntegrityStatus;
  sync_operation: SyncOperationType | null;
  sync_status: SyncStatus | null;
  updated_at: number;
  tags: CloudTrack['tags'];
};

type CloudFolderDTO = {
  id: string;
  provider_type: string;
  cloud_folder_path: string;
  local_folder_path: string;
  sync_status: FolderSyncStatus;
  track_count: number;
  pending_sync_count: number;
};

type StorageUsageDTO = {
  used_bytes: number;
  total_bytes: number;
  last_sync: number;
};

type QueueItemDTO = {
  id: string;
  cloud_track_id: string;
  file_name: string;
  operation: SyncOperationType;
  status: SyncStatus;
  created_at: number;
  updated_at: number;
  provider_type: string;
};

type QueueStatsDTO = {
  pending_count: number;
  in_progress_count: number;
  completed_count: number;
  failed_count: number;
};

type CloudPageDataDTO = {
  folders: CloudFolderDTO[];
  tracks: CloudTrackDTO[];
  storage: StorageUsageDTO;
  queue_items: QueueItemDTO[];
  queue_stats: QueueStatsDTO;
  selected_folder_id: string | null;
};

type TrackSyncDetailsDTO = {
  track: CloudTrackDTO;
  sync_history: Array<{
    timestamp: number;
    operation: SyncOperationType;
    old_hash: string | null;
    new_hash: string | null;
    status: SyncStatus;
  }>;
  current_operation: QueueItemDTO | null;
};

// Command types (to be implemented later with Tauri)
type CloudCommands = {
  getCloudPageData: (folderId?: string) => Promise<CloudPageDataDTO>;
  forceSyncFolder: (folderId: string) => Promise<void>;
  setSyncPaused: (paused: boolean) => Promise<void>;
  retryFailedItems: (folderId?: string) => Promise<void>;
  getTrackSyncDetails: (trackId: string) => Promise<TrackSyncDetailsDTO>;
};

// Mock data
const mockStorageUsage: StorageUsageDTO = {
  used_bytes: 12.5 * 1024 * 1024 * 1024,
  total_bytes: 50 * 1024 * 1024 * 1024,
  last_sync: Date.now() - 5 * 60 * 1000, // 5 minutes ago
};

const mockFolders: CloudFolderDTO[] = [
  {
    id: '1',
    provider_type: 'dropbox',
    cloud_folder_path: '/Music',
    local_folder_path: '/home/user/Music',
    sync_status: FolderSyncStatus.Synced,
    track_count: 5,
    pending_sync_count: 0,
  },
  {
    id: '2',
    provider_type: 'dropbox',
    cloud_folder_path: '/Playlists',
    local_folder_path: '/home/user/Playlists',
    sync_status: FolderSyncStatus.NeedsAttention,
    track_count: 3,
    pending_sync_count: 2,
  },
  {
    id: '3',
    provider_type: 'gdrive',
    cloud_folder_path: '/Albums',
    local_folder_path: '/home/user/Albums',
    sync_status: FolderSyncStatus.Empty,
    track_count: 0,
    pending_sync_count: 0,
  },
];

const mockTracks: CloudTrackDTO[] = [
  {
    id: '1',
    file_name: 'song1.mp3',
    relative_path: 'song1.mp3',
    integrity_status: TrackIntegrityStatus.Complete,
    sync_operation: null,
    sync_status: null,
    updated_at: Date.now(),
    tags: {
      title: 'Song 1',
      album: 'Album 1',
      artists: ['Artist 1'],
      genres: ['Rock'],
      year: 2024,
      duration: 180,
      track_no: 1,
      track_of: 12,
      disk_no: 1,
      disk_of: 1,
    },
  },
  {
    id: '2',
    file_name: 'local_only.mp3',
    relative_path: 'local_only.mp3',
    integrity_status: TrackIntegrityStatus.LocalOnly,
    sync_operation: SyncOperationType.Upload,
    sync_status: SyncStatus.InProgress,
    updated_at: Date.now() - 3600000,
    tags: {
      title: 'Local Only Track',
      album: 'Local Album',
      artists: ['Local Artist'],
      genres: ['Jazz'],
      year: 2024,
      duration: 240,
      track_no: 1,
      track_of: 10,
      disk_no: 1,
      disk_of: 1,
    },
  },
  {
    id: '3',
    file_name: 'cloud_only.mp3',
    relative_path: 'cloud_only.mp3',
    integrity_status: TrackIntegrityStatus.CloudOnly,
    sync_operation: SyncOperationType.Download,
    sync_status: SyncStatus.Pending,
    updated_at: Date.now() - 7200000,
    tags: null,
  },
  {
    id: '4',
    file_name: 'conflict.mp3',
    relative_path: 'conflict.mp3',
    integrity_status: TrackIntegrityStatus.OutOfSync,
    sync_operation: null,
    sync_status: null,
    updated_at: Date.now() - 300000,
    tags: {
      title: 'Conflict Track',
      album: 'Conflict Album',
      artists: ['Conflict Artist'],
      genres: ['Electronic'],
      year: 2024,
      duration: 195,
      track_no: 5,
      track_of: 12,
      disk_no: 1,
      disk_of: 1,
    },
  },
];

const mockQueueItems: QueueItemDTO[] = [
  {
    id: '1',
    cloud_track_id: '2',
    file_name: 'local_only.mp3',
    operation: SyncOperationType.Upload,
    status: SyncStatus.InProgress,
    created_at: Date.now() - 60000,
    updated_at: Date.now(),
    provider_type: 'dropbox',
  },
  {
    id: '2',
    cloud_track_id: '3',
    file_name: 'cloud_only.mp3',
    operation: SyncOperationType.Download,
    status: SyncStatus.Pending,
    created_at: Date.now(),
    updated_at: Date.now(),
    provider_type: 'dropbox',
  },
];

const mockQueueStats: QueueStatsDTO = {
  pending_count: 1,
  in_progress_count: 1,
  completed_count: 5,
  failed_count: 0,
};

// Helper function to get status display info
const getStatusDisplay = (track: CloudTrackDTO): { icon: string; text: string; color: string } => {
  // If track is currently syncing, show sync status
  if (track.sync_operation && track.sync_status) {
    switch (track.sync_status) {
      case SyncStatus.InProgress:
        return {
          icon: track.sync_operation === SyncOperationType.Upload ? '⬆️' : '⬇️',
          text: track.sync_operation === SyncOperationType.Upload ? 'Uploading' : 'Downloading',
          color: 'var(--info-color)',
        };
      case SyncStatus.Pending:
        return {
          icon: '⏳',
          text: 'Queued',
          color: 'var(--text-muted)',
        };
      case SyncStatus.Failed:
        return {
          icon: '❌',
          text: 'Failed',
          color: 'var(--danger-color)',
        };
      default:
        break;
    }
  }

  // Otherwise show integrity status
  switch (track.integrity_status) {
    case TrackIntegrityStatus.Complete:
      return { icon: '✓', text: 'Synced', color: 'var(--success-color)' };
    case TrackIntegrityStatus.LocalOnly:
      return { icon: '💻', text: 'Local Only', color: 'var(--warning-color)' };
    case TrackIntegrityStatus.CloudOnly:
      return { icon: '☁️', text: 'Cloud Only', color: 'var(--warning-color)' };
    case TrackIntegrityStatus.OutOfSync:
      return { icon: '⚠️', text: 'Out of Sync', color: 'var(--warning-color)' };
    case TrackIntegrityStatus.Missing:
      return { icon: '❌', text: 'Missing', color: 'var(--danger-color)' };
    case TrackIntegrityStatus.NotMapped:
      return { icon: '❓', text: 'Not Mapped', color: 'var(--danger-color)' };
    default:
      return { icon: '❓', text: 'Unknown', color: 'var(--danger-color)' };
  }
};

export default function ViewCloud() {
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);

  return (
    <div className={styles.container}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.status}>
          <div className={styles.statusIcon}>✓</div>
          <div className={styles.statusText}>
            <div>In Sync</div>
            <div className={styles.lastSync}>
              Last sync: {new Date(mockStorageUsage.last_sync).toLocaleString()}
            </div>
          </div>
        </div>
        <div className={styles.storage}>
          <div className={styles.storageText}>
            {(mockStorageUsage.used_bytes / 1024 / 1024 / 1024).toFixed(1)} GB used of{' '}
            {(mockStorageUsage.total_bytes / 1024 / 1024 / 1024).toFixed(1)} GB
          </div>
          <div className={styles.storageBar}>
            <div
              className={styles.storageBarFill}
              style={{
                width: `${(mockStorageUsage.used_bytes / mockStorageUsage.total_bytes) * 100}%`,
              }}
            />
          </div>
        </div>
        <div className={styles.actions}>
          <button>Force Sync All</button>
        </div>
      </div>

      <div className={styles.content}>
        {/* Sidebar */}
        <div className={styles.sidebar}>
          <h3>Cloud Folders</h3>
          <ul className={styles.folderList}>
            {mockFolders.map(folder => (
              <li
                key={folder.id}
                className={selectedFolder === folder.id ? styles.selected : ''}
                onClick={() => setSelectedFolder(folder.id)}
              >
                <span className={styles.folderIcon}>📁</span>
                <span className={styles.folderName}>{folder.cloud_folder_path}</span>
                <span className={styles.folderStatus}>
                  {folder.pending_sync_count > 0 && (
                    <span className={styles.badge}>{folder.pending_sync_count}</span>
                  )}
                  {folder.sync_status === FolderSyncStatus.Synced && '✓'}
                  {folder.sync_status === FolderSyncStatus.NeedsAttention && '⚠️'}
                  {folder.sync_status === FolderSyncStatus.Syncing && '↻'}
                  {folder.sync_status === FolderSyncStatus.Empty && ''}
                </span>
              </li>
            ))}
          </ul>
        </div>

        {/* Main Content */}
        <div className={styles.main}>
          <div className={styles.toolbar}>
            <div className={styles.filters}>
              <select>
                <option>All Files</option>
                <option>Local Only</option>
                <option>Cloud Only</option>
                <option>Out of Sync</option>
                <option>Syncing</option>
              </select>
              <input type="text" placeholder="Search files..." />
            </div>
          </div>

          <table className={styles.trackList}>
            <thead>
              <tr>
                <th>Name</th>
                <th>Status</th>
                <th>Path</th>
                <th>Last Updated</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {mockTracks.map(track => {
                const status = getStatusDisplay(track);
                return (
                  <tr key={track.id}>
                    <td>{track.tags?.title || track.file_name}</td>
                    <td>
                      <span className={styles.syncStatus} style={{ color: status.color }}>
                        {status.icon} {status.text}
                      </span>
                    </td>
                    <td>{track.relative_path}</td>
                    <td>{new Date(track.updated_at).toLocaleString()}</td>
                    <td>
                      <button>⋮</button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* Queue Status Bar */}
      <div className={styles.queueStatus}>
        <div className={styles.queueTabs}>
          <button className={styles.active}>
            Current ({mockQueueStats.in_progress_count + mockQueueStats.pending_count})
          </button>
          <button>Completed ({mockQueueStats.completed_count})</button>
          <button>Failed ({mockQueueStats.failed_count})</button>
        </div>
        <div className={styles.queueList}>
          {mockQueueItems.map(item => (
            <div key={item.id} className={styles.queueItem}>
              <span>
                {item.operation === SyncOperationType.Upload ? '⬆️' : '⬇️'} {item.file_name}
              </span>
              <span>
                {item.status === SyncStatus.InProgress ? 'In Progress' : 'Queued'}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

