import { useEffect, useMemo, useState } from 'react';
import type { SortBy, SortOrder, UnifiedTrack } from '../generated/typings';
import {
  filterUnifiedTracks,
  getUnifiedSortOrder,
  sortUnifiedTracks,
  getLocationTypeWithFileCheck,
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
  const [checkedTracks, setCheckedTracks] = useState<UnifiedTrack[]>([]);

  // First filter and sort tracks
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

  // Then check file existence for each track
  useEffect(() => {
    const checkFiles = async () => {
      const tracksWithFileCheck = await Promise.all(
        filteredTracks.map(async (track) => ({
          ...track,
          location_type: await getLocationTypeWithFileCheck(track),
        }))
      );
      setCheckedTracks(tracksWithFileCheck);
    };

    checkFiles();
  }, [filteredTracks]);

  // Update the footer status based on the displayed tracks
  useEffect(() => {
    const tracksForStatus = checkedTracks.map(track => ({
      id: track.local_track_id || track.cloud_track_id || '',
      path: track.local_path || track.cloud_relative_path || '',
      duration: track.duration,
      artists: track.artists || [],
      composers: [], // Default empty array for composers
      album_artists: [], // Default empty array for album artists
      album: track.album,
      title: track.title,
      genres: track.genres || [],
      year: track.year,
      date: null, // Default null for date
      track_no: track.track_no,
      track_of: track.track_of,
      disk_no: track.disk_no,
      disk_of: track.disk_of,
      bitrate: null, // Default null for bitrate
      sampling_rate: null, // Default null for sampling rate
      channels: null, // Default null for channels
      encoder: null, // Default null for encoder
    }));
    
    libraryAPI.setTracksStatus(tracksForStatus);

    return () => {
      libraryAPI.setTracksStatus(null);
    };
  }, [checkedTracks, libraryAPI.setTracksStatus]);

  return checkedTracks;
}
