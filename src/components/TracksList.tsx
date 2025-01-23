import { DndContext, type DragEndEvent } from '@dnd-kit/core';
import { restrictToVerticalAxis } from '@dnd-kit/modifiers';
import {
  SortableContext,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { useVirtualizer } from '@tanstack/react-virtual';
import {
  Menu,
  MenuItem,
  PredefinedMenuItem,
  Submenu,
} from '@tauri-apps/api/menu';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import type React from 'react';
import { useCallback, useEffect, useRef } from 'react';
import Keybinding from 'react-keybinding-component';
import { useSearchParams } from 'react-router';

import type { Config, Playlist, Track } from '../generated/typings';
import { useLibraryAPI } from '../stores/useLibraryStore';
import { usePlayerAPI } from '../stores/usePlayerStore';
import { useTrackSelection } from '../hooks/useTrackSelection';
import { useTracksKeyboardNavigation } from '../hooks/useTracksKeyboardNavigation';
import { useTracksContextMenu } from '../hooks/useTracksContextMenu';
import { useScrollRestoration } from '../hooks/useScrollRestoration';
import useDndSensors from '../hooks/useDnDSensors';
import useInvalidate from '../hooks/useInvalidate';
import TrackRow from './TrackRow';
import TracksListHeader from './TracksListHeader';
import styles from './TracksList.module.css';

const ROW_HEIGHT = 30;
const ROW_HEIGHT_COMPACT = 24;
const DND_MODIFIERS = [restrictToVerticalAxis];

// --------------------------------------------------------------------------
// TrackList
// --------------------------------------------------------------------------

type Props = {
  type: string;
  tracks: Track[];
  tracksDensity: Config['track_view_density'];
  trackPlayingID: string | null;
  playlists: Playlist[];
  currentPlaylist?: string;
  reorderable?: boolean;
  onReorder?: (tracks: Track[]) => void;
};

export default function TracksList(props: Props) {
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

  // APIs and utilities
  const playerAPI = usePlayerAPI();
  const libraryAPI = useLibraryAPI();
  const invalidate = useInvalidate();
  const sensors = useDndSensors();

  // Scrollable element for the virtual list + virtualizer
  const scrollableRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: tracks.length,
    overscan: 20,
    scrollPaddingEnd: 22, // Height of the track list header
    getScrollElement: () => scrollableRef.current,
    estimateSize: () => {
      switch (tracksDensity) {
        case 'compact':
          return ROW_HEIGHT_COMPACT;
        default:
          return ROW_HEIGHT;
      }
    },
    getItemKey: (index) => tracks[index].id,
  });

  useScrollRestoration(scrollableRef);

  // Track selection hook
  const {
    selectedTracks,
    setSelectedTracks,
    selectTrack,
    selectTrackClick,
    selectAllTracks,
  } = useTrackSelection({ tracks });

  // Keyboard navigation hook
  const { onKey } = useTracksKeyboardNavigation({
    tracks,
    selectedTracks,
    setSelectedTracks,
    virtualizer,
    onStartPlayback: (trackID) => playerAPI.start(tracks, trackID),
    selectAllTracks,
  });

  // Context menu hook
  const { showContextMenu } = useTracksContextMenu({
    tracks,
    selectedTracks,
    type,
    playlists,
    currentPlaylist,
    playerAPI,
    libraryAPI,
    invalidate,
  });

  // Highlight playing track and scroll to it
  useEffect(() => {
    if (shouldJumpToPlayingTrack && trackPlayingID) {
      setSearchParams(undefined);
      setSelectedTracks(new Set([trackPlayingID]));

      const playingTrackIndex = tracks.findIndex(
        (track) => track.id === trackPlayingID,
      );

      if (playingTrackIndex >= 0) {
        setTimeout(() => {
          // avoid conflict with scroll restoration
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

  /**
   * Playlist tracks re-order events handlers
   */
  const onDragEnd = useCallback(
    (event: DragEndEvent) => {
      const {
        active, // dragged item
        over, // on which item it was dropped
      } = event;

      // The item was dropped either nowhere, or on the same item
      if (over == null || active.id === over.id || !onReorder) {
        return;
      }

      const activeIndex = tracks.findIndex((track) => track.id === active.id);
      const overIndex = tracks.findIndex((track) => track.id === over.id);

      const newTracks = [...tracks];

      const movedTrack = newTracks.splice(activeIndex, 1)[0]; // Remove active track
      newTracks.splice(overIndex, 0, movedTrack); // Move it to where the user dropped it

      onReorder(newTracks);
    },
    [onReorder, tracks],
  );

  const startPlayback = useCallback(
    async (trackID: string) => {
      playerAPI.start(tracks, trackID);
    },
    [tracks, playerAPI],
  );

  return (
    <DndContext
      onDragEnd={onDragEnd}
      id="dnd-playlist"
      modifiers={DND_MODIFIERS}
      sensors={sensors}
    >
      <div className={styles.tracksList}>
        <Keybinding onKey={onKey} preventInputConflict />
        {/* Scrollable element */}
        <div ref={scrollableRef} className={styles.tracksListScroller}>
          <TracksListHeader enableSort={type === 'library'} />

          {/* The large inner element to hold all of the items */}
          <div
            className={styles.tracksListRows}
            style={{
              height: `${virtualizer.getTotalSize()}px`,
              width: '100%',
              position: 'relative',
            }}
          >
            <SortableContext
              items={tracks}
              strategy={verticalListSortingStrategy}
            >
              {/* Only the visible items in the virtualizer, manually positioned to be in view */}
              {virtualizer.getVirtualItems().map((virtualItem) => {
                const track = tracks[virtualItem.index];
                return (
                  <TrackRow
                    key={virtualItem.key}
                    selected={selectedTracks.has(track.id)}
                    track={track}
                    isPlaying={trackPlayingID === track.id}
                    index={virtualItem.index}
                    onMouseDown={selectTrack}
                    onClick={selectTrackClick}
                    onContextMenu={showContextMenu}
                    onDoubleClick={startPlayback}
                    draggable={reorderable}
                    style={{
                      position: 'absolute',
                      left: 0,
                      width: '100%',
                      height: `${virtualItem.size}px`,
                      // Intentionally not translateY, as it would create another paint
                      // layer where every row would cover elements from the previous one.
                      // This would typically prevent the drop effect to be properly displayed
                      // when reordering a playlist
                      top: `${virtualItem.start}px`,
                      zIndex: 1,
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
