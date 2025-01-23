import { useCallback } from 'react';
import type { Track } from '../generated/typings';
import { isCtrlKey } from '../lib/utils-events';
import type { Virtualizer } from '@tanstack/react-virtual';

interface UseTracksKeyboardNavigationProps {
  tracks: Track[];
  selectedTracks: Set<string>;
  setSelectedTracks: (tracks: Set<string>) => void;
  virtualizer: Virtualizer<HTMLDivElement, Element>;
  onStartPlayback: (trackID: string) => void;
  selectAllTracks: () => void;
}

interface UseTracksKeyboardNavigationReturn {
  onKey: (e: KeyboardEvent) => void;
}

export function useTracksKeyboardNavigation({
  tracks,
  selectedTracks,
  setSelectedTracks,
  virtualizer,
  onStartPlayback,
  selectAllTracks,
}: UseTracksKeyboardNavigationProps): UseTracksKeyboardNavigationReturn {
  const onUp = useCallback(
    (index: number, tracks: Track[], shiftKeyPressed: boolean) => {
      const addedIndex = Math.max(0, index - 1);

      // Add to the selection if shift key is pressed
      let newSelected = selectedTracks;

      if (shiftKeyPressed)
        newSelected = new Set([tracks[addedIndex].id, ...selectedTracks]);
      else newSelected = new Set([tracks[addedIndex].id]);

      setSelectedTracks(newSelected);
      virtualizer.scrollToIndex(addedIndex);
    },
    [selectedTracks, virtualizer, setSelectedTracks],
  );

  const onDown = useCallback(
    (index: number, tracks: Track[], shiftKeyPressed: boolean) => {
      const addedIndex = Math.min(tracks.length - 1, index + 1);
      // Add to the selection if shift key is pressed
      let newSelected: Set<string>;
      if (shiftKeyPressed)
        newSelected = new Set([...selectedTracks, tracks[addedIndex].id]);
      else newSelected = new Set([tracks[addedIndex].id]);
      setSelectedTracks(newSelected);
      virtualizer.scrollToIndex(addedIndex);
    },
    [selectedTracks, virtualizer, setSelectedTracks],
  );

  const onEnter = useCallback(
    async (index: number, tracks: Track[]) => {
      if (index !== -1) onStartPlayback(tracks[index].id);
    },
    [onStartPlayback],
  );

  const onKey = useCallback(
    (e: KeyboardEvent) => {
      const firstSelectedTrackID = tracks.findIndex((track) =>
        selectedTracks.has(track.id),
      );

      switch (e.key) {
        case 'a':
          if (isCtrlKey(e)) {
            selectAllTracks();
            e.preventDefault();
          }
          break;

        case 'ArrowUp':
          e.preventDefault();
          onUp(firstSelectedTrackID, tracks, e.shiftKey);
          break;

        case 'ArrowDown': {
          const lastSelectedTrackID = tracks.findLastIndex((track) =>
            selectedTracks.has(track.id),
          );
          e.preventDefault();
          onDown(lastSelectedTrackID, tracks, e.shiftKey);
          break;
        }

        case 'Enter':
          e.preventDefault();
          void onEnter(firstSelectedTrackID, tracks);
          break;

        default:
          break;
      }
    },
    [onUp, onDown, onEnter, selectAllTracks, selectedTracks, tracks],
  );

  return { onKey };
} 