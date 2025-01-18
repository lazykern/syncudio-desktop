import React, { JSX, useState, useCallback, useRef, memo, useMemo } from 'react';
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
import * as Checkbox from '@radix-ui/react-checkbox';
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
import { useCloudFolders, useCloudFolderDetails, useQueueItems, useQueueStats, useSyncMutations, type FolderWithDetails } from '../hooks/useCloudQueries';
import styles from './cloud.module.css';
import { useVirtualizer } from '@tanstack/react-virtual';

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

// Helper function to get location display info
const getLocationDisplay = (locationState: TrackLocationState): { icon: JSX.Element; text: string; color: string } => {
  switch (locationState) {
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

// Helper function to get sync display info
const getSyncDisplay = (operation: SyncOperationType | null, status: SyncStatus | null): { icon: JSX.Element; text: string; color: string } | null => {
  if (!operation || !status) return null;

  if (status === 'in_progress') {
    return {
      icon: operation === 'upload' ? <RiUploadCloud2Line /> : <RiDownloadCloud2Line />,
      text: operation === 'upload' ? 'Uploading' : 'Downloading',
      color: 'var(--info-color)',
    };
  }
  if (status === 'pending') {
    return {
      icon: <RiTimeLine />,
      text: 'Queued',
      color: 'var(--text-muted)',
    };
  }
  if (typeof status === 'object' && 'failed' in status) {
    return {
      icon: <RiCloseLine />,
      text: 'Failed',
      color: 'var(--danger-color)',
    };
  }
  return null;
};

// Helper function to get folder status display
const getFolderStatusDisplay = (status: FolderSyncStatus): { icon: JSX.Element; text: string; color: string } => {
  switch (status) {
    case 'synced':
      return { icon: <RiCheckLine />, text: 'Synced', color: 'var(--success-color)' };
    case 'syncing':
      return { icon: <RiRefreshLine />, text: 'Syncing', color: 'var(--info-color)' };
    case 'needs_attention':
      return { icon: <RiErrorWarningLine />, text: 'Needs Attention', color: 'var(--warning-color)' };
    case 'empty':
      return { icon: <RiCloudLine />, text: 'Empty', color: 'var(--text-muted)' };
  }
};

type QueueTab = 'current' | 'completed' | 'failed';

// Create a virtualized row component
const VirtualRow = memo(function VirtualRow({ 
  track, 
  isSelected, 
  onSelect 
}: { 
  track: CloudTrackDTO; 
  isSelected: boolean; 
  onSelect: (id: string) => void;
}) {
  const locationDisplay = useMemo(() => getLocationDisplay(track.location_state), [track.location_state]);
  const syncDisplay = useMemo(() => getSyncDisplay(track.sync_operation, track.sync_status), [track.sync_operation, track.sync_status]);

  return (
    <div className={styles.virtualRow}>
      <div className={styles.cell}>
        <Checkbox.Root
          className={styles.checkbox}
          checked={isSelected}
          onCheckedChange={() => onSelect(track.id)}
        >
          <Checkbox.Indicator className={styles.checkboxIndicator}>
            <RiCheckLine />
          </Checkbox.Indicator>
        </Checkbox.Root>
      </div>
      <div className={styles.cell}>{track.tags?.title || track.file_name}</div>
      <div className={styles.cell}>
        <span style={{ color: locationDisplay.color }}>
          <span className={styles.statusIcon}>{locationDisplay.icon}</span>
          {locationDisplay.text}
        </span>
      </div>
      <div className={styles.cell}>
        {syncDisplay && (
          <span style={{ color: syncDisplay.color }}>
            <span className={styles.statusIcon}>{syncDisplay.icon}</span>
            {syncDisplay.text}
          </span>
        )}
      </div>
      <div className={styles.cell}>{track.relative_path}</div>
      <div className={styles.cell}>{new Date(track.updated_at).toLocaleString()}</div>
      <div className={styles.cell}>
        <button className={styles.actionButton}>
          <FaEllipsisVertical />
        </button>
      </div>
    </div>
  );
});

export default function ViewCloud() {
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [activeQueueTab, setActiveQueueTab] = useState<QueueTab>('current');
  const [locationFilter, setLocationFilter] = useState<TrackLocationState | 'all'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTracks, setSelectedTracks] = useState<Set<string>>(new Set());
  
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
  const { uploadMutation, downloadMutation, isLoading: isSyncing } = useSyncMutations(selectedFolder || undefined);

  // Filter tracks based on location state and search query
  const filteredTracks = folderDetails?.tracks.filter(track => {
    // Apply location filter
    if (locationFilter !== 'all' && track.location_state !== locationFilter) {
      return false;
    }

    // Apply search filter
    if (searchQuery) {
      const searchLower = searchQuery.toLowerCase();
      const fileName = track.file_name.toLowerCase();
      const relativePath = track.relative_path.toLowerCase();
      const title = track.tags?.title?.toLowerCase() || '';
      const album = track.tags?.album?.toLowerCase() || '';
      const artists = track.tags?.artists?.join(' ').toLowerCase() || '';

      return fileName.includes(searchLower) ||
        relativePath.includes(searchLower) ||
        title.includes(searchLower) ||
        album.includes(searchLower) ||
        artists.includes(searchLower);
    }

    return true;
  }) || [];

  const handleTrackSelect = (trackId: string) => {
    setSelectedTracks(prev => {
      const next = new Set(prev);
      if (next.has(trackId)) {
        next.delete(trackId);
      } else {
        next.add(trackId);
      }
      return next;
    });
  };

  const handleSelectAll = () => {
    if (selectedTracks.size === filteredTracks.length) {
      setSelectedTracks(new Set());
    } else {
      setSelectedTracks(new Set(filteredTracks.map(t => t.id)));
    }
  };

  const handleSyncSelected = async () => {
    if (selectedTracks.size === 0) return;
    
    // Group tracks by sync direction
    const uploadTracks: string[] = [];
    const downloadTracks: string[] = [];

    // Get selected tracks from filteredTracks
    const selectedTrackObjects = filteredTracks.filter(track => selectedTracks.has(track.id));

    // Determine direction based on location_state
    for (const track of selectedTrackObjects) {
      switch (track.location_state) {
        case 'local_only':
          uploadTracks.push(track.id);
          break;
        case 'cloud_only':
          downloadTracks.push(track.id);
          break;
        case 'out_of_sync':
          // For out of sync, we prioritize local version
          uploadTracks.push(track.id);
          break;
        // Skip complete, missing, and not_mapped states
      }
    }

    try {
      // Add tracks to appropriate queues
      if (uploadTracks.length > 0) {
        await uploadMutation.mutateAsync(uploadTracks);
      }
      if (downloadTracks.length > 0) {
        await downloadMutation.mutateAsync(downloadTracks);
      }
      setSelectedTracks(new Set());
    } catch (error) {
      console.error('Failed to sync tracks:', error);
      // Here you could add a toast notification for the error
    }
  };

  const handleForceSyncAll = async () => {
    if (selectedFolder && folderDetails) {
      const uploadTracks: string[] = [];
      const downloadTracks: string[] = [];

      // Determine direction for all tracks
      for (const track of folderDetails.tracks) {
        switch (track.location_state) {
          case 'local_only':
            uploadTracks.push(track.id);
            break;
          case 'cloud_only':
            downloadTracks.push(track.id);
            break;
          case 'out_of_sync':
            // For out of sync, we prioritize local version
            uploadTracks.push(track.id);
            break;
          // Skip complete, missing, and not_mapped states
        }
      }

      try {
        // Add tracks to appropriate queues
        if (uploadTracks.length > 0) {
          await uploadMutation.mutateAsync(uploadTracks);
        }
        if (downloadTracks.length > 0) {
          await downloadMutation.mutateAsync(downloadTracks);
        }
      } catch (error) {
        console.error('Failed to sync all tracks:', error);
        // Here you could add a toast notification for the error
      }
    }
  };

  // Filter queue items based on active tab
  const filteredQueueItems = queueItems.filter(item => {
    switch (activeQueueTab) {
      case 'current':
        return item.status === 'in_progress' || item.status === 'pending';
      case 'completed':
        return item.status === 'completed';
      case 'failed':
        return typeof item.status === 'object' && 'failed' in item.status;
      default:
        return false;
    }
  });

  // Get the selected folder's status display
  const selectedFolderStatus = folderDetails 
    ? getFolderStatusDisplay(folderDetails.sync_status)
    : { icon: '', text: 'Select a folder', color: 'var(--text-muted)' };

  const parentRef = useRef<HTMLDivElement>(null);

  const rowVirtualizer = useVirtualizer({
    count: filteredTracks.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 48,
    overscan: 5,
  });

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
          {selectedTracks.size > 0 && (
            <button 
              onClick={handleSyncSelected}
              className={styles.syncButton}
              disabled={isSyncing}
            >
              {isSyncing ? 'Syncing...' : `Sync Selected (${selectedTracks.size})`}
            </button>
          )}
          <button 
            onClick={handleForceSyncAll}
            disabled={!selectedFolder || isSyncing}
            className={styles.syncButton}
          >
            {isSyncing ? 'Syncing...' : 'Force Sync All'}
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
                  <select 
                    value={locationFilter}
                    onChange={(e) => setLocationFilter(e.target.value as TrackLocationState | 'all')}
                  >
                    <option value="all">All Files</option>
                    <option value="complete">Synced</option>
                    <option value="local_only">Local Only</option>
                    <option value="cloud_only">Cloud Only</option>
                    <option value="out_of_sync">Out of Sync</option>
                    <option value="missing">Missing</option>
                    <option value="not_mapped">Not Mapped</option>
              </select>
                  <input 
                    type="text" 
                    placeholder="Search files..." 
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                  />
            </div>
          </div>

          <div className={styles.trackList}>
            <div className={styles.tableHeader}>
              <div className={styles.headerCell}>
                <Checkbox.Root
                  className={styles.checkbox}
                  checked={selectedTracks.size === filteredTracks.length && filteredTracks.length > 0}
                  onCheckedChange={handleSelectAll}
                >
                  <Checkbox.Indicator className={styles.checkboxIndicator}>
                    <RiCheckLine />
                  </Checkbox.Indicator>
                </Checkbox.Root>
              </div>
              <div className={styles.headerCell}>Name</div>
              <div className={styles.headerCell}>Location</div>
              <div className={styles.headerCell}>Sync Status</div>
              <div className={styles.headerCell}>Path</div>
              <div className={styles.headerCell}>Last Updated</div>
              <div className={styles.headerCell}>Actions</div>
            </div>

            {filteredTracks.length === 0 ? (
              <div style={{ textAlign: 'center', padding: '2rem', color: 'var(--text-muted)' }}>
                No tracks match the current filters
              </div>
            ) : (
              <div className={styles.trackListContainer} ref={parentRef}>
                <div
                  style={{
                    height: `${rowVirtualizer.getTotalSize()}px`,
                    width: '100%',
                    position: 'relative',
                  }}
                >
                  {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                    const track = filteredTracks[virtualRow.index];
                    return (
                      <div
                        key={track.id}
                        style={{
                          position: 'absolute',
                          top: 0,
                          left: 0,
                          width: '100%',
                          transform: `translateY(${virtualRow.start}px)`,
                        }}
                      >
                        <VirtualRow
                          track={track}
                          isSelected={selectedTracks.has(track.id)}
                          onSelect={handleTrackSelect}
                        />
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </div>
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
          <button 
            className={`${styles.queueTab} ${activeQueueTab === 'current' ? styles.active : ''}`}
            onClick={() => setActiveQueueTab('current')}
          >
            <span className={styles.queueTabIcon}>
              {queueStats.in_progress_count > 0 ? (
                <RiRefreshLine />
              ) : (
                <RiTimeLine />
              )}
            </span>
            Current ({queueStats.in_progress_count + queueStats.pending_count})
          </button>
          <button 
            className={`${styles.queueTab} ${activeQueueTab === 'completed' ? styles.active : ''}`}
            onClick={() => setActiveQueueTab('completed')}
          >
            <span className={styles.queueTabIcon}>
              <RiCheckLine />
            </span>
            Completed ({queueStats.completed_count})
          </button>
          <button 
            className={`${styles.queueTab} ${activeQueueTab === 'failed' ? styles.active : ''}`}
            onClick={() => setActiveQueueTab('failed')}
          >
            <span className={styles.queueTabIcon}>
              <RiCloseLine />
            </span>
            Failed ({queueStats.failed_count})
          </button>
        </div>
        <div className={styles.queueList}>
          {filteredQueueItems.map(item => (
            <div key={item.id} className={styles.queueItem}>
              <span className={styles.queueItemName}>
                {item.operation === 'upload' ? <RiUploadCloud2Line /> : <RiDownloadCloud2Line />} {item.file_name}
              </span>
              <span className={styles.queueItemStatus}>
                {typeof item.status === 'object' && 'failed' in item.status ? (
                  <span className={styles.queueItemError}>
                    Failed: {item.status.failed.error} (Attempts: {item.status.failed.attempts})
                  </span>
                ) : item.status === 'in_progress' ? (
                  <span className={styles.queueItemProgress}>
                    <RiRefreshLine /> In Progress
                  </span>
                ) : item.status === 'completed' ? (
                  <span className={styles.queueItemSuccess}>
                    <RiCheckLine /> Completed
                  </span>
                ) : (
                  <span className={styles.queueItemPending}>
                    <RiTimeLine /> Queued
                  </span>
                )}
              </span>
            </div>
          ))}
          {filteredQueueItems.length === 0 && (
            <div className={styles.queueEmpty}>
              {activeQueueTab === 'current' && 'No active sync operations'}
              {activeQueueTab === 'completed' && 'No completed sync operations'}
              {activeQueueTab === 'failed' && 'No failed sync operations'}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

