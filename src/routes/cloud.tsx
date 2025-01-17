import React, { JSX, useState } from 'react';
import {
  FaDropbox,
  FaGoogleDrive,
  FaFolder,
  FaEllipsisVertical,
} from 'react-icons/fa6';
import {
  RiCheckLine,
  RiErrorWarningLine,
  RiQuestionLine,
  RiCloudLine,
  RiUploadCloud2Line,
  RiDownloadCloud2Line,
  RiTimeLine,
  RiCloseLine,
  RiComputerLine,
  RiCloudOffLine,
  RiRefreshLine,
} from 'react-icons/ri';
import type {
  CloudTrackDTO,
  CloudFolder,
  CloudFolderSyncDetailsDTO,
  QueueItemDTO,
  QueueStatsDTO,
  TrackLocationState,
  SyncOperationType,
  SyncStatus,
  FolderSyncStatus,
} from '../generated/typings';
import { cloudSync } from '../lib/cloud-sync';
import { useCloudFolders, useCloudFolderDetails, useQueueItems, useQueueStats, type FolderWithDetails } from '../hooks/useCloudQueries';
import styles from './cloud.module.css';

// Helper function to get provider icon
const getProviderIcon = (providerType: string) => {
  switch (providerType.toLowerCase()) {
    case 'dropbox':
      return <FaDropbox />;
    case 'gdrive':
      return <FaGoogleDrive />;
    default:
      return <FaFolder />;
  }
};

// Helper function to get status display info
const getStatusDisplay = (track: CloudTrackDTO): { icon: JSX.Element; text: string; color: string } => {
  // If track is currently syncing, show sync status
  if (track.sync_operation && track.sync_status) {
    if (track.sync_status === 'in_progress') {
      return {
        icon: track.sync_operation === 'upload' ? <RiUploadCloud2Line /> : <RiDownloadCloud2Line />,
        text: track.sync_operation === 'upload' ? 'Uploading' : 'Downloading',
        color: 'var(--info-color)',
      };
    }
    if (track.sync_status === 'pending') {
      return {
        icon: <RiTimeLine />,
        text: 'Queued',
        color: 'var(--text-muted)',
      };
    }
    if (typeof track.sync_status === 'object' && 'failed' in track.sync_status) {
      return {
        icon: <RiCloseLine />,
        text: 'Failed',
        color: 'var(--danger-color)',
      };
    }
  }

  // Otherwise show integrity status
  switch (track.location_state) {
    case 'complete':
      return { icon: <RiCheckLine />, text: 'Synced', color: 'var(--success-color)' };
    case 'local_only':
      return { icon: <RiComputerLine />, text: 'Local Only', color: 'var(--warning-color)' };
    case 'cloud_only':
      return { icon: <RiCloudLine />, text: 'Cloud Only', color: 'var(--warning-color)' };
    case 'out_of_sync':
      return { icon: <RiErrorWarningLine />, text: 'Out of Sync', color: 'var(--warning-color)' };
    case 'missing':
      return { icon: <RiCloudOffLine />, text: 'Missing', color: 'var(--danger-color)' };
    case 'not_mapped':
      return { icon: <RiQuestionLine />, text: 'Not Mapped', color: 'var(--danger-color)' };
    default:
      return { icon: <RiQuestionLine />, text: 'Unknown', color: 'var(--danger-color)' };
  }
};

// Helper function to get folder status display
const getFolderStatusDisplay = (status: FolderSyncStatus): { icon: JSX.Element; text: string; color: string } => {
  switch (status) {
    case 'synced':
      return { icon: <RiCheckLine />, text: 'Synced', color: 'var(--success-color)' };
    case 'syncing':
      return { icon: <RiRefreshLine className={styles.spinningIcon} />, text: 'Syncing', color: 'var(--info-color)' };
    case 'needs_attention':
      return { icon: <RiErrorWarningLine />, text: 'Needs Attention', color: 'var(--warning-color)' };
    case 'empty':
      return { icon: <RiCloudLine />, text: 'Empty', color: 'var(--text-muted)' };
  }
};

export default function ViewCloud() {
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  
  // Use React Query hooks
  const { data: folders = [] } = useCloudFolders();
  const { data: folderDetails } = useCloudFolderDetails(selectedFolder);
  const { data: queueItems = [] } = useQueueItems(selectedFolder || undefined);
  const { data: queueStats = {
    pending_count: 0,
    in_progress_count: 0,
    completed_count: 0,
    failed_count: 0,
  } } = useQueueStats(selectedFolder || undefined);

  const handleForceSyncAll = async () => {
    if (selectedFolder) {
      await cloudSync.forceSyncFolder(selectedFolder);
      // React Query will automatically refetch the data
    }
  };

  // Get the selected folder's status display
  const selectedFolderStatus = folderDetails 
    ? getFolderStatusDisplay(folderDetails.sync_status)
    : { icon: '', text: 'Select a folder', color: 'var(--text-muted)' };

  return (
    <div className={styles.container}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.status}>
          <div className={styles.statusIcon} style={{ color: selectedFolderStatus.color }}>
            {selectedFolderStatus.icon}
          </div>
          <div className={styles.statusText}>
            <div>{selectedFolderStatus.text}</div>
            {folderDetails && folderDetails.pending_sync_count > 0 && (
              <div className={styles.pendingCount}>
                {folderDetails.pending_sync_count} items pending sync
            </div>
            )}
          </div>
        </div>
        <div className={styles.actions}>
          <button 
            onClick={handleForceSyncAll}
            disabled={!selectedFolder}
            className={styles.syncButton}
          >
            Force Sync All
          </button>
        </div>
      </div>

      <div className={styles.content}>
        {/* Sidebar */}
        <div className={styles.sidebar}>
          <h3>Cloud Folders</h3>
          <ul className={styles.folderList}>
            {folders.map((folder: FolderWithDetails) => {
              const status = folder.details ? getFolderStatusDisplay(folder.details.sync_status) : null;
              
              return (
              <li
                key={folder.id}
                  className={`${styles.folderItem} ${selectedFolder === folder.id ? styles.selected : ''}`}
                onClick={() => setSelectedFolder(folder.id)}
              >
                  <span className={styles.folderIcon}>
                    {getProviderIcon(folder.provider_type)}
                  </span>
                <span className={styles.folderName}>{folder.cloud_folder_path}</span>
                  {status && folder.details && (
                    <span className={styles.folderStatus} style={{ color: status.color }}>
                      {folder.details.pending_sync_count > 0 && (
                        <span className={styles.badge}>{folder.details.pending_sync_count}</span>
                      )}
                      {status.icon}
                    </span>
                  )}
              </li>
              );
            })}
          </ul>
        </div>

        {/* Main Content */}
        <div className={styles.main}>
          {selectedFolder && folderDetails ? (
            <>
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
                  {folderDetails.tracks.map(track => {
                const status = getStatusDisplay(track);
                return (
                  <tr key={track.id}>
                    <td>{track.tags?.title || track.file_name}</td>
                    <td>
                      <span style={{ color: track.sync_operation ? 'var(--text-muted)' : status.color }}>
                        <span className={styles.statusIcon}>{status.icon}</span>
                        {status.text}
                      </span>
                    </td>
                    <td>
                      {track.sync_operation && (
                        <span style={{ color: status.color }}>
                          <span className={styles.statusIcon}>{status.icon}</span>
                          {status.text}
                        </span>
                      )}
                    </td>
                    <td>{track.relative_path}</td>
                    <td>{new Date(track.updated_at).toLocaleString()}</td>
                    <td>
                          <button className={styles.actionButton}>
                            <FaEllipsisVertical />
                          </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
            </>
          ) : (
            <div className={styles.noSelection}>
              <div className={styles.noSelectionIcon}>
                <RiCloudLine />
              </div>
              <h2>Select a Cloud Folder</h2>
              <p>Choose a folder from the sidebar to view and manage its tracks</p>
              {folders.length === 0 && (
                <p className={styles.noFolders}>
                  No cloud folders configured. Add one in the <a href="#/settings/cloud">settings</a>.
                </p>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Queue Status Bar */}
      <div className={styles.queueStatus}>
        <div className={styles.queueTabs}>
          <button className={styles.active}>
            Current ({queueStats.in_progress_count + queueStats.pending_count})
          </button>
          <button>Completed ({queueStats.completed_count})</button>
          <button>Failed ({queueStats.failed_count})</button>
        </div>
        <div className={styles.queueList}>
          {queueItems.map(item => (
            <div key={item.id} className={styles.queueItem}>
              <span className={styles.queueItemName}>
                {item.operation === 'upload' ? <RiUploadCloud2Line /> : <RiDownloadCloud2Line />} {item.file_name}
              </span>
              <span className={styles.queueItemStatus}>
                {item.status === 'in_progress' ? 'In Progress' : 'Queued'}
              </span>
            </div>
          ))}
          {queueItems.length === 0 && (
            <div className={styles.queueEmpty}>No active sync operations</div>
          )}
        </div>
      </div>
    </div>
  );
}

