import { DndContext, type DragEndEvent } from '@dnd-kit/core';
import { restrictToVerticalAxis } from '@dnd-kit/modifiers';
import {
  SortableContext,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { useVirtualizer } from '@tanstack/react-virtual';
import type React from 'react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import Keybinding from 'react-keybinding-component';
import { useSearchParams } from 'react-router';
import { useQueryClient } from '@tanstack/react-query';

import type { Config, Playlist, UnifiedTrack, TrackDownloadedPayload } from '../generated/typings';
import { useLibraryAPI } from '../stores/useLibraryStore';
import { usePlayerAPI } from '../stores/usePlayerStore';
import { useTrackSelection } from '../hooks/useTrackSelection';
import { useTracksKeyboardNavigation } from '../hooks/useTracksKeyboardNavigation';
import { useTracksContextMenu } from '../hooks/useTracksContextMenu';
import { useScrollRestoration } from '../hooks/useScrollRestoration';
import useDndSensors from '../hooks/useDnDSensors';
import useInvalidate from '../hooks/useInvalidate';
import { useSyncMutations } from '../hooks/useCloudQueries';
import { checkFileExists } from '../lib/utils-unified-tracks';
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
  
  const [checkedTracks, setCheckedTracks] = useState<UnifiedTrack[]>([]);
  const [localTracks, setLocalTracks] = useState<UnifiedTrack[]>([]);
  
  // Check file existence for all tracks
  useEffect(() => {
    async function checkTracks() {
      const checked = await Promise.all(
        tracks.map(async (track) => {
          const exists = track.local_path ? await checkFileExists(track.local_path) : false;
          return {
            ...track,
            _exists: exists // Add internal flag for existence
          };
        })
      );
      setCheckedTracks(checked);
      
      // Filter tracks that exist locally
      const local = checked.filter(t => t._exists);
      setLocalTracks(local);
    }
    
    checkTracks();
  }, [tracks]);

  // Listen for track-downloaded events to update tracks
  useEffect(() => {
    const unlisten = listen<TrackDownloadedPayload>('track-downloaded', async (event) => {
      // Update specific track in cache without full refetch
      await queryClient.setQueryData(['unified-tracks'], (oldData: UnifiedTrack[] | undefined) => {
        if (!oldData) return oldData;
        
        return oldData.map(track => {
          if (track.cloud_track_id === event.payload.cloud_track_id) {
            return {
              ...track,
              local_track_id: event.payload.local_track_id,
              local_path: `${event.payload.sync_folder_id}/${event.payload.relative_path}`
            };
          }
          return track;
        });
      });

      // Also invalidate to ensure consistency
      await queryClient.invalidateQueries({ queryKey: ['unified-tracks'] });
    });

    // Return cleanup function
    return () => {
      unlisten.then(unsubscribe => unsubscribe());
    };
  }, [queryClient]);

  const scrollableRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: checkedTracks.length,
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
    getItemKey: (index) => checkedTracks[index].local_track_id || checkedTracks[index].cloud_track_id || index.toString(),
  });

  useScrollRestoration(scrollableRef);

  // Only tracks that exist locally can be selected
  const nonCloudTracks = localTracks.map(t => ({
    id: t.local_track_id || t.cloud_track_id || '',
    path: t.local_path || '',
    title: t.title,
    album: t.album,
    artists: t.artists || [],
    composers: [], // Default empty array
    album_artists: [], // Default empty array
    genres: t.genres || [],
    duration: t.duration,
    year: t.year,
    date: null, // Default null
    track_no: t.track_no,
    track_of: t.track_of,
    disk_no: t.disk_no,
    disk_of: t.disk_of,
    bitrate: null, // Default null
    sampling_rate: null, // Default null
    channels: null, // Default null
    encoder: null, // Default null
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

      const playingTrackIndex = checkedTracks.findIndex(
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
    checkedTracks,
    virtualizer,
    setSelectedTracks,
  ]);

  const onDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event;

      if (over == null || active.id === over.id || !onReorder) {
        return;
      }

      const activeIndex = checkedTracks.findIndex((track) => 
        (track.local_track_id || track.cloud_track_id) === active.id
      );
      const overIndex = checkedTracks.findIndex((track) => 
        (track.local_track_id || track.cloud_track_id) === over.id
      );

      const newTracks = [...checkedTracks];
      const movedTrack = newTracks.splice(activeIndex, 1)[0];
      newTracks.splice(overIndex, 0, movedTrack);

      onReorder(newTracks);
    },
    [onReorder, checkedTracks],
  );

  async function handleTrackPlay(trackId: string) {
    const track = checkedTracks.find(t => (t.local_track_id || t.cloud_track_id) === trackId);
    if (!track) return;

    // Create local tracks list excluding non-existent tracks
    const playableTracks = localTracks.map(t => ({
      id: t.local_track_id || t.cloud_track_id || '',
      path: t.local_path || '',
      title: t.title,
      album: t.album,
      artists: t.artists || [],
      composers: [], // Default empty array
      album_artists: [], // Default empty array
      genres: t.genres || [],
      duration: t.duration,
      year: t.year,
      date: null, // Default null
      track_no: t.track_no,
      track_of: t.track_of,
      disk_no: t.disk_no,
      disk_of: t.disk_of,
      bitrate: null, // Default null
      sampling_rate: null, // Default null
      channels: null, // Default null
      encoder: null, // Default null
    }));

    const exists = track.local_path ? await checkFileExists(track.local_path) : false;
    if (!exists) {
      if (track.cloud_track_id && track.cloud_folder_id) {
        try {
          await downloadMutation.mutateAsync({
            trackIds: [track.cloud_track_id],
            folderId: track.cloud_folder_id,
          });
          toastsAPI.add('success', `Downloading "${track.title}"`);
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
              items={checkedTracks.map(t => t.local_track_id || t.cloud_track_id || '')}
              strategy={verticalListSortingStrategy}
            >
              {virtualizer.getVirtualItems().map((virtualItem) => {
                const track = checkedTracks[virtualItem.index];
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
                    draggable={reorderable && (track.local_track_id !== null)}
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
