import { useState } from 'react';
import type {
  CloudTrackDTO,
  CloudFolderDTO,
  QueueItemDTO,
  QueueStatsDTO,
  StorageUsageDTO,
  CloudPageDataDTO,
  TrackSyncDetailsDTO,
  TrackLocationState,
  SyncOperationType,
  SyncStatus,
  FolderSyncStatus,
} from '../generated/typings';
import styles from './cloud.module.css';

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
  used_bytes: BigInt(12.5 * 1024 * 1024 * 1024),
  total_bytes: BigInt(50 * 1024 * 1024 * 1024),
  last_sync: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
};

const mockFolders: CloudFolderDTO[] = [
  {
    id: '1',
    provider_type: 'dropbox',
    cloud_folder_path: '/Music',
    local_folder_path: '/home/user/Music',
    sync_status: 'synced',
    track_count: 5,
    pending_sync_count: 0,
  },
  {
    id: '2',
    provider_type: 'dropbox',
    cloud_folder_path: '/Playlists',
    local_folder_path: '/home/user/Playlists',
    sync_status: 'needs_attention',
    track_count: 3,
    pending_sync_count: 2,
  },
  {
    id: '3',
    provider_type: 'gdrive',
    cloud_folder_path: '/Albums',
    local_folder_path: '/home/user/Albums',
    sync_status: 'empty',
    track_count: 0,
    pending_sync_count: 0,
  },
];

const mockTracks: CloudTrackDTO[] = [
  {
    id: '1',
    file_name: 'song1.mp3',
    relative_path: 'song1.mp3',
    location_state: 'complete',
    sync_operation: null,
    sync_status: null,
    updated_at: new Date(Date.now()).toISOString(),
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
    location_state: 'local_only',
    sync_operation: 'upload',
    sync_status: 'in_progress',
    updated_at: new Date(Date.now() - 3600000).toISOString(),
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
    location_state: 'cloud_only',
    sync_operation: 'download',
    sync_status: 'pending',
    updated_at: new Date(Date.now() - 7200000).toISOString(),
    tags: null,
  },
  {
    id: '4',
    file_name: 'conflict.mp3',
    relative_path: 'conflict.mp3',
    location_state: 'out_of_sync',
    sync_operation: null,
    sync_status: null,
    updated_at: new Date(Date.now() - 300000).toISOString(),
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
    operation: 'upload',
    status: 'in_progress',
    created_at: new Date(Date.now() - 60000).toISOString(),
    updated_at: new Date(Date.now()).toISOString(),
    provider_type: 'dropbox',
  },
  {
    id: '2',
    cloud_track_id: '3',
    file_name: 'cloud_only.mp3',
    operation: 'download',
    status: 'pending',
    created_at: new Date(Date.now()).toISOString(),
    updated_at: new Date(Date.now()).toISOString(),
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
    if (track.sync_status === 'in_progress') {
      return {
        icon: track.sync_operation === 'upload' ? '⬆️' : '⬇️',
        text: track.sync_operation === 'upload' ? 'Uploading' : 'Downloading',
        color: 'var(--info-color)',
      };
    }
    if (track.sync_status === 'pending') {
      return {
        icon: '⏳',
        text: 'Queued',
        color: 'var(--text-muted)',
      };
    }
    if (typeof track.sync_status === 'object' && 'failed' in track.sync_status) {
      return {
        icon: '❌',
        text: 'Failed',
        color: 'var(--danger-color)',
      };
    }
  }

  // Otherwise show integrity status
  switch (track.location_state) {
    case 'complete':
      return { icon: '✓', text: 'Synced', color: 'var(--success-color)' };
    case 'local_only':
      return { icon: '💻', text: 'Local Only', color: 'var(--warning-color)' };
    case 'cloud_only':
      return { icon: '☁️', text: 'Cloud Only', color: 'var(--warning-color)' };
    case 'out_of_sync':
      return { icon: '⚠️', text: 'Out of Sync', color: 'var(--warning-color)' };
    case 'missing':
      return { icon: '❌', text: 'Missing', color: 'var(--danger-color)' };
    case 'not_mapped':
      return { icon: '❓', text: 'Not Mapped', color: 'var(--danger-color)' };
    default:
      return { icon: '❓', text: 'Unknown', color: 'var(--danger-color)' };
  }
};

// Helper function to convert bigint to number safely
const bigintToNumber = (value: bigint): number => {
  // This is safe for our use case since we're dealing with timestamps and file sizes
  // that won't exceed Number.MAX_SAFE_INTEGER
  return Number(value);
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
            {(bigintToNumber(mockStorageUsage.used_bytes) / 1024 / 1024 / 1024).toFixed(1)} GB used of{' '}
            {(bigintToNumber(mockStorageUsage.total_bytes) / 1024 / 1024 / 1024).toFixed(1)} GB
          </div>
          <div className={styles.storageBar}>
            <div
              className={styles.storageBarFill}
              style={{
                width: `${(bigintToNumber(mockStorageUsage.used_bytes) / bigintToNumber(mockStorageUsage.total_bytes)) * 100}%`,
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
                  {folder.sync_status === 'synced' && '✓'}
                  {folder.sync_status === 'needs_attention' && '⚠️'}
                  {folder.sync_status === 'syncing' && '↻'}
                  {folder.sync_status === 'empty' && ''}
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
                <th>Location</th>
                <th>Sync Status</th>
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
                      <span style={{ color: track.sync_operation ? 'var(--text-muted)' : status.color }}>
                        {track.location_state === 'complete' && '✓ Both'}
                        {track.location_state === 'local_only' && '💻 Local Only'}
                        {track.location_state === 'cloud_only' && '☁️ Cloud Only'}
                        {track.location_state === 'out_of_sync' && '⚠️ Out of Sync'}
                        {track.location_state === 'missing' && '❌ Missing'}
                        {track.location_state === 'not_mapped' && '❓ Not Mapped'}
                      </span>
                    </td>
                    <td>
                      {track.sync_operation && (
                        <span style={{ color: status.color }}>
                          {track.sync_operation === 'upload' ? '⬆️' : '⬇️'}{' '}
                          {track.sync_status === 'in_progress' ? 'In Progress' : 'Queued'}
                        </span>
                      )}
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
                {item.operation === 'upload' ? '⬆️' : '⬇️'} {item.file_name}
              </span>
              <span>
                {item.status === 'in_progress' ? 'In Progress' : 'Queued'}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

