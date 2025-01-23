import { DndContext, type DragEndEvent } from '@dnd-kit/core';
import { restrictToVerticalAxis } from '@dnd-kit/modifiers';
import {
  SortableContext,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { useVirtualizer } from '@tanstack/react-virtual';
import type React from 'react';
import { useCallback, useEffect, useRef } from 'react';
import Keybinding from 'react-keybinding-component';
import { useSearchParams } from 'react-router';
import { useQueryClient } from '@tanstack/react-query';

import type { Config, Playlist, UnifiedTrack } from '../generated/typings';
import { useLibraryAPI } from '../stores/useLibraryStore';
import { usePlayerAPI } from '../stores/usePlayerStore';
import { useTrackSelection } from '../hooks/useTrackSelection';
import { useTracksKeyboardNavigation } from '../hooks/useTracksKeyboardNavigation';
import { useTracksContextMenu } from '../hooks/useTracksContextMenu';
import { useScrollRestoration } from '../hooks/useScrollRestoration';
import useDndSensors from '../hooks/useDnDSensors';
import useInvalidate from '../hooks/useInvalidate';
import { useSyncMutations } from '../hooks/useCloudQueries';
import UnifiedTrackRow from './UnifiedTrackRow';
import TracksListHeader from './TracksListHeader';
import styles from './TracksList.module.css';
import { useToastsAPI } from '../stores/useToastsStore';

const ROW_HEIGHT = 30;
const ROW_HEIGHT_COMPACT = 24;
const DND_MODIFIERS = [restrictToVerticalAxis];

type Props = {
  type: string;
  tracks: UnifiedTrack[];
  tracksDensity: Config['track_view_density'];
  trackPlayingID: string | null;
  playlists: Playlist[];
  currentPlaylist?: string;
  reorderable?: boolean;
  onReorder?: (tracks: UnifiedTrack[]) => void;
};

export default function UnifiedTracksList(props: Props) {
  const {
    tracks,
    type,
    tracksDensity,
    trackPlayingID,
    reorderable,
    currentPlaylist,
    onReorder,
    playlists,
  } = props;

  const [searchParams, setSearchParams] = useSearchParams();
  const shouldJumpToPlayingTrack =
    searchParams.get('jump_to_playing_track') === 'true';

  const { downloadMutation } = useSyncMutations();
  const playerAPI = usePlayerAPI();
  const libraryAPI = useLibraryAPI();
  const toastsAPI = useToastsAPI();
  const invalidate = useInvalidate();
  const queryClient = useQueryClient();
  const sensors = useDndSensors();
  
  // Listen for track-downloaded events to refresh the list
  useEffect(() => {
    const handleTrackDownloaded = async () => {
      // Invalidate unified tracks query to refresh the list
      await queryClient.invalidateQueries({ queryKey: ['unified-tracks'] });
    };

    window.addEventListener('track-downloaded', handleTrackDownloaded);
    return () => window.removeEventListener('track-downloaded', handleTrackDownloaded);
  }, [queryClient]);

  const scrollableRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: tracks.length,
    overscan: 20,
    scrollPaddingEnd: 22,
    getScrollElement: () => scrollableRef.current,
    estimateSize: () => {
      switch (tracksDensity) {
        case 'compact':
          return ROW_HEIGHT_COMPACT;
        default:
          return ROW_HEIGHT;
      }
    },
    getItemKey: (index) => tracks[index].local_track_id || tracks[index].cloud_track_id || index.toString(),
  });

  useScrollRestoration(scrollableRef);

  // Only non-cloud tracks can be selected
  const nonCloudTracks = tracks.filter(t => t.location_type !== 'cloud').map(t => ({
    id: t.local_track_id || t.cloud_track_id || '',
    path: t.local_path || '',
    title: t.title,
    album: t.album,
    artists: t.artists || [],
    genres: t.genres || [],
    duration: t.duration,
    year: t.year,
    track_no: t.track_no,
    track_of: t.track_of,
    disk_no: t.disk_no,
    disk_of: t.disk_of,
    blake3_hash: t.blake3_hash
  }));

  const {
    selectedTracks,
    setSelectedTracks,
    selectTrack,
    selectTrackClick,
    selectAllTracks,
  } = useTrackSelection({ 
    tracks: nonCloudTracks
  });

  const { onKey } = useTracksKeyboardNavigation({
    tracks: nonCloudTracks,
    selectedTracks,
    setSelectedTracks,
    virtualizer,
    onStartPlayback: handleTrackPlay,
    selectAllTracks,
  });

  const { showContextMenu } = useTracksContextMenu({
    tracks: nonCloudTracks,
    selectedTracks,
    type,
    playlists,
    currentPlaylist,
    playerAPI,
    libraryAPI,
    invalidate,
  });

  useEffect(() => {
    if (shouldJumpToPlayingTrack && trackPlayingID) {
      setSearchParams(undefined);
      setSelectedTracks(new Set([trackPlayingID]));

      const playingTrackIndex = tracks.findIndex(
        (track) => (track.local_track_id || track.cloud_track_id) === trackPlayingID,
      );

      if (playingTrackIndex >= 0) {
        setTimeout(() => {
          virtualizer.scrollToIndex(playingTrackIndex, { behavior: 'smooth' });
        }, 0);
      }
    }
  }, [
    shouldJumpToPlayingTrack,
    setSearchParams,
    trackPlayingID,
    tracks,
    virtualizer,
    setSelectedTracks,
  ]);

  const onDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event;

      if (over == null || active.id === over.id || !onReorder) {
        return;
      }

      const activeIndex = tracks.findIndex((track) => 
        (track.local_track_id || track.cloud_track_id) === active.id
      );
      const overIndex = tracks.findIndex((track) => 
        (track.local_track_id || track.cloud_track_id) === over.id
      );

      const newTracks = [...tracks];
      const movedTrack = newTracks.splice(activeIndex, 1)[0];
      newTracks.splice(overIndex, 0, movedTrack);

      onReorder(newTracks);
    },
    [onReorder, tracks],
  );

  async function handleTrackPlay(trackId: string) {
    const track = tracks.find(t => (t.local_track_id || t.cloud_track_id) === trackId);
    if (!track) return;

    // Create local tracks list excluding cloud-only tracks
    const playableTracks = tracks
      .filter(t => t.location_type !== 'cloud')
      .map(t => ({
        id: t.local_track_id || t.cloud_track_id || '',
        path: t.local_path || '',
        title: t.title,
        album: t.album,
        artists: t.artists || [],
        genres: t.genres || [],
        duration: t.duration,
        year: t.year,
        track_no: t.track_no,
        track_of: t.track_of,
        disk_no: t.disk_no,
        disk_of: t.disk_of,
        blake3_hash: t.blake3_hash
    }));

    if (track.location_type === 'cloud') {
      if (track.cloud_track_id && track.cloud_folder_id) {
        try {
          await downloadMutation.mutateAsync({
            trackIds: [track.cloud_track_id],
            folderId: track.cloud_folder_id,
          });
          toastsAPI.add('success', `Downloading "${track.title}"`);
          // Don't start playing yet - wait for download to complete
          // The cloud sync worker will emit track-downloaded event which will update the UI
        } catch (error) {
          console.error('Failed to download track:', error);
          toastsAPI.add('danger', `Failed to download "${track.title}"`);
        }
      }
    } else {
      playerAPI.start(playableTracks, trackId);
    }
  }

  return (
    <DndContext
      onDragEnd={onDragEnd}
      id="dnd-playlist"
      modifiers={DND_MODIFIERS}
      sensors={sensors}
    >
      <div className={styles.tracksList}>
        <Keybinding onKey={onKey} preventInputConflict />
        <div ref={scrollableRef} className={styles.tracksListScroller}>
          <TracksListHeader enableSort={type === 'library'} />

          <div
            className={styles.tracksListRows}
            style={{
              height: `${virtualizer.getTotalSize()}px`,
              width: '100%',
              position: 'relative',
            }}
          >
            <SortableContext
              items={tracks.map(t => t.local_track_id || t.cloud_track_id || '')}
              strategy={verticalListSortingStrategy}
            >
              {virtualizer.getVirtualItems().map((virtualItem) => {
                const track = tracks[virtualItem.index];
                const trackId = track.local_track_id || track.cloud_track_id || '';
                return (
                  <UnifiedTrackRow
                    key={virtualItem.key}
                    selected={selectedTracks.has(trackId)}
                    track={track}
                    isPlaying={trackPlayingID === trackId}
                    index={virtualItem.index}
                    onMouseDown={selectTrack}
                    onClick={selectTrackClick}
                    onContextMenu={showContextMenu}
                    onDoubleClick={handleTrackPlay}
                    draggable={reorderable && track.location_type !== 'cloud'}
                    style={{
                      position: 'absolute',
                      left: 0,
                      width: '100%',
                      height: `${virtualItem.size}px`,
                      top: `${virtualItem.start}px`,
                      zIndex: 1
                    }}
                  />
                );
              })}
            </SortableContext>
          </div>
        </div>
      </div>
    </DndContext>
  );
}
