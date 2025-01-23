import { useEffect, useMemo } from 'react';
import type { SortBy, SortOrder, UnifiedTrack } from '../generated/typings';
import {
  filterUnifiedTracks,
  getUnifiedSortOrder,
  sortUnifiedTracks,
} from '../lib/utils-unified-tracks';
import { stripAccents } from '../lib/utils-library';
import useLibraryStore, { useLibraryAPI } from '../stores/useLibraryStore';

/**
 * Filter and Sort a list of unified tracks depending on the user preferences and search
 * IMPORTANT: can only be used ONCE per view, as it has side effects
 */
export default function useFilteredUnifiedTracks(
  tracks: UnifiedTrack[],
  sortBy?: SortBy,
  sortOrder?: SortOrder,
): UnifiedTrack[] {
  const search = useLibraryStore((state) => stripAccents(state.search));
  const libraryAPI = useLibraryAPI();

  const filteredTracks = useMemo(() => {
    let searchedTracks = filterUnifiedTracks(tracks, search);

    if (sortBy && sortOrder) {
      // sorting being a costly operation, do it after filtering, ignore it if not needed
      searchedTracks = sortUnifiedTracks(
        searchedTracks,
        getUnifiedSortOrder(sortBy),
        sortOrder,
      );
    }

    return searchedTracks;
  }, [tracks, search, sortBy, sortOrder]);

  // Update the footer status based on the displayed tracks
  useEffect(() => {
    const tracksForStatus = filteredTracks.map(track => ({
      id: track.local_track_id || track.cloud_track_id || '',
      path: track.local_path || track.cloud_relative_path || '',
      duration: track.duration,
      artists: track.artists || [],
      album: track.album,
      title: track.title,
      genres: track.genres || [],
      year: track.year,
      track_no: track.track_no,
      track_of: track.track_of,
      disk_no: track.disk_no,
      disk_of: track.disk_of,
      blake3_hash: track.blake3_hash
    }));
    
    libraryAPI.setTracksStatus(tracksForStatus);

    return () => {
      libraryAPI.setTracksStatus(null);
    };
  }, [filteredTracks, libraryAPI.setTracksStatus]);

  return filteredTracks;
}
