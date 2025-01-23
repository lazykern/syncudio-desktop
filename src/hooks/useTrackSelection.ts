import { useState, useCallback } from 'react';
import type { Track, UnifiedTrack } from '../generated/typings';
import { keyboardSelect } from '../lib/utils-list';

interface UseTrackSelectionProps {
  tracks: Track[] | UnifiedTrack[];
  preventCloudOnlySelect?: boolean;
}

interface UseTrackSelectionReturn {
  selectedTracks: Set<string>;
  setSelectedTracks: (tracks: Set<string>) => void;
  selectTrack: (event: React.MouseEvent, trackID: string) => void;
  selectTrackClick: (event: React.MouseEvent | React.KeyboardEvent, trackID: string) => void;
  selectAllTracks: () => void;
}

export function useTrackSelection({ tracks, preventCloudOnlySelect }: UseTrackSelectionProps): UseTrackSelectionReturn {
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

      // For unified tracks, prevent selecting cloud-only tracks if preventCloudOnlySelect is true
      if (preventCloudOnlySelect && 'location_type' in tracks[0]) {
        const track = tracks.find((t) => 
          (t as UnifiedTrack).local_track_id === trackID || 
          (t as UnifiedTrack).cloud_track_id === trackID
        ) as UnifiedTrack;

        if (track && track.location_type === 'cloud') {
          return;
        }
      }

      setSelectedTracks(keyboardSelect(
        tracks.map(t => ({
          id: 'local_track_id' in t ? (t.local_track_id || t.cloud_track_id || '') : t.id,
          path: 'local_path' in t ? (t.local_path || '') : t.path,
        })), 
        selectedTracks, 
        trackID, 
        event
      ));
    },
    [tracks, selectedTracks, preventCloudOnlySelect],
  );

  const selectTrackClick = useCallback(
    (event: React.MouseEvent | React.KeyboardEvent, trackID: string) => {
      if (
        !event.metaKey &&
        !event.ctrlKey &&
        !event.shiftKey &&
        selectedTracks.has(trackID)
      ) {
        // For unified tracks, prevent selecting cloud-only tracks if preventCloudOnlySelect is true
        if (preventCloudOnlySelect && 'location_type' in tracks[0]) {
          const track = tracks.find((t) => 
            (t as UnifiedTrack).local_track_id === trackID || 
            (t as UnifiedTrack).cloud_track_id === trackID
          ) as UnifiedTrack;

          if (track && track.location_type === 'cloud') {
            return;
          }
        }

        setSelectedTracks(new Set([trackID]));
      }
    },
    [selectedTracks, tracks, preventCloudOnlySelect],
  );

  const selectAllTracks = useCallback(() => {
    if (preventCloudOnlySelect && 'location_type' in tracks[0]) {
      // Only select non-cloud tracks
      const nonCloudTracks = tracks.filter((t) => 
        (t as UnifiedTrack).location_type !== 'cloud'
      );

      setSelectedTracks(new Set(nonCloudTracks.map((track) => 
        (track as UnifiedTrack).local_track_id || 
        (track as UnifiedTrack).cloud_track_id || ''
      )));
    } else {
      setSelectedTracks(new Set(tracks.map((track) => 
        'local_track_id' in track ? 
          (track.local_track_id || track.cloud_track_id || '') : 
          track.id
      )));
    }
  }, [tracks, preventCloudOnlySelect]);

  return {
    selectedTracks,
    setSelectedTracks,
    selectTrack,
    selectTrackClick,
    selectAllTracks,
  };
}
