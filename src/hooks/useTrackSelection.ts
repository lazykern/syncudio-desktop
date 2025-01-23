import { useState, useCallback } from 'react';
import type { Track } from '../generated/typings';
import { keyboardSelect } from '../lib/utils-list';

interface UseTrackSelectionProps {
  tracks: Track[];
}

interface UseTrackSelectionReturn {
  selectedTracks: Set<string>;
  setSelectedTracks: (tracks: Set<string>) => void;
  selectTrack: (event: React.MouseEvent, trackID: string) => void;
  selectTrackClick: (event: React.MouseEvent | React.KeyboardEvent, trackID: string) => void;
  selectAllTracks: () => void;
}

export function useTrackSelection({ tracks }: UseTrackSelectionProps): UseTrackSelectionReturn {
  const [selectedTracks, setSelectedTracks] = useState<Set<string>>(new Set());

  const selectTrack = useCallback(
    (event: React.MouseEvent, trackID: string) => {
      // To allow selection drag-and-drop, we need to prevent track selection
      // when selecting a track that is already selected
      if (
        selectedTracks.has(trackID) &&
        !event.metaKey &&
        !event.ctrlKey &&
        !event.shiftKey
      ) {
        return;
      }

      setSelectedTracks(keyboardSelect(tracks, selectedTracks, trackID, event));
    },
    [tracks, selectedTracks],
  );

  const selectTrackClick = useCallback(
    (event: React.MouseEvent | React.KeyboardEvent, trackID: string) => {
      if (
        !event.metaKey &&
        !event.ctrlKey &&
        !event.shiftKey &&
        selectedTracks.has(trackID)
      ) {
        setSelectedTracks(new Set([trackID]));
      }
    },
    [selectedTracks],
  );

  const selectAllTracks = useCallback(() => {
    setSelectedTracks(new Set(tracks.map((track) => track.id)));
  }, [tracks]);

  return {
    selectedTracks,
    setSelectedTracks,
    selectTrack,
    selectTrackClick,
    selectAllTracks,
  };
} 