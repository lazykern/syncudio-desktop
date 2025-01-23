import { useSortable } from '@dnd-kit/sortable';
import cx from 'classnames';
import type React from 'react';

import type { UnifiedTrack, TrackSyncStatusDTO } from '../generated/typings';
import useFormattedDuration from '../hooks/useFormattedDuration';
import PlayingIndicator from './PlayingIndicator';
import { useTrackSyncStatus } from '../hooks/useCloudQueries';

import styles from './TrackRow.module.css';
import cellStyles from './TracksListHeader.module.css';

type Props = {
  selected: boolean;
  track: UnifiedTrack;
  index: number;
  isPlaying?: boolean;
  onDoubleClick?: (trackID: string) => void;
  onMouseDown?: (
    event: React.MouseEvent,
    trackID: string,
    index: number,
  ) => void;
  onContextMenu?: (event: React.MouseEvent, index: number) => void;
  onClick?: (
    event: React.MouseEvent | React.KeyboardEvent,
    trackID: string,
  ) => void;
  draggable?: boolean;
  style?: React.CSSProperties;
};

function getSyncDescription(status?: TrackSyncStatusDTO | null): string {
  if (!status) {
    return 'Double click to download';
  }

  if (status.sync_operation === 'download') {
    const syncStatus = status.sync_status;
    if (!syncStatus) {
      return 'Double click to download';
    }
    
    if (syncStatus === 'in_progress') {
      return 'Downloading...';
    }
    if (syncStatus === 'completed') {
      return 'Download completed';
    }
    if (syncStatus === 'pending') {
      return 'Download pending...';
    }
    if (typeof syncStatus === 'object' && 'failed' in syncStatus) {
      return `Download failed - ${syncStatus.failed.error} (${syncStatus.failed.attempts} attempts) - Double click to retry`;
    }
  }

  return 'Double click to download';
}

export default function UnifiedTrackRow(props: Props) {
  const {
    track,
    index,
    selected,
    draggable,
    onMouseDown,
    onClick,
    onContextMenu,
    onDoubleClick,
  } = props;

  const duration = useFormattedDuration(track.duration);
  const trackId = track.local_track_id || track.cloud_track_id || '';
  const isCloudOnly = track.location_type === 'cloud';
  
  const { data: syncStatus } = useTrackSyncStatus(track.cloud_track_id || '');
  const isDownloading = 
    syncStatus?.sync_operation === 'download' && 
    syncStatus?.sync_status === 'in_progress';

  const syncDescription = getSyncDescription(syncStatus);

  // Drag-and-Drop for playlists
  const {
    attributes,
    listeners,
    setNodeRef,
    isDragging,
    isOver,
    activeIndex,
    overIndex,
  } = useSortable({
    id: trackId,
    disabled: !draggable || isCloudOnly,
    data: {
      type: 'playlist-track',
      index,
      isCloudOnly,
    },
  });

  const trackClasses = cx(styles.track, {
    [styles.selected]: selected,
    [styles.even]: index % 2 === 0,
    [styles.isDragging]: isDragging,
    [styles.isOver]: isOver,
    [styles.isAbove]: isOver && overIndex < activeIndex,
    [styles.isBelow]: isOver && overIndex > activeIndex,
    [styles.cloudOnly]: isCloudOnly && !isDownloading,
    [styles.downloading]: isDownloading,
  });

  const title = isCloudOnly ? `${track.title} - ${syncDescription}` : track.title;

  return (
    <div
      className={trackClasses}
      onDoubleClick={() => onDoubleClick?.(trackId)}
      onMouseDown={(e) => onMouseDown?.(e, trackId, index)}
      onClick={(e) => onClick?.(e, trackId)}
      onKeyDown={(e) => {
        if (e.key === 'Enter') {
          onClick?.(e, trackId);
        }
      }}
      onContextMenu={(e) => onContextMenu?.(e, index)}
      aria-selected={selected}
      {...(props.isPlaying ? { 'data-is-playing': true } : {})}
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      style={props.style}
      title={title}
      data-cloud-only={isCloudOnly}
      data-downloading={isDownloading}
    >
      <div className={`${styles.cell} ${cellStyles.cellTrackPlaying}`}>
        {props.isPlaying ? <PlayingIndicator /> : null}
      </div>
      <div className={`${styles.cell} ${cellStyles.cellTrack}`}>
        {track.title}
      </div>
      <div className={`${styles.cell} ${cellStyles.cellDuration}`}>
        {duration}
      </div>
      <div className={`${styles.cell} ${cellStyles.cellArtist}`}>
        {track.artists?.join(', ') || ''}
      </div>
      <div className={`${styles.cell} ${cellStyles.cellAlbum}`}>
        {track.album}
      </div>
      <div className={`${styles.cell} ${cellStyles.cellGenre}`}>
        {track.genres?.join(', ') || ''}
      </div>
    </div>
  );
}
