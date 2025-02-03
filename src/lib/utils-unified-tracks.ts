import orderBy from 'lodash-es/orderBy';
import type { UnifiedTrack, SortBy, SortOrder } from '../generated/typings';
import { stripAccents } from './utils-library';
import { parseDuration } from '../hooks/useFormattedDuration';
import { plural } from './localization';
import { invoke } from '@tauri-apps/api/core';

/**
 * Check if a file exists in the filesystem
 */
export async function checkFileExists(path: string): Promise<boolean> {
  if (!path) return false;
  
  try {
    return await invoke('plugin:cloud|check_file_exists', { path });
  } catch (error) {
    // Log the error for debugging
    console.warn(`Failed to check file existence for path: ${path}`, error);
    
    // Return false for forbidden paths or any other errors
    return false;
  }
}

/**
 * Get the location type for a unified track by checking actual file existence
 */
export async function getLocationTypeWithFileCheck(track: UnifiedTrack): Promise<'local' | 'cloud' | 'both'> {
  if (track.local_track_id && track.cloud_track_id) {
    return 'both';
  } else if (track.local_track_id) {
    return 'local';
  } else if (track.cloud_track_id) {
    return 'cloud';
  }
  // Fallback to cloud if no IDs are present (shouldn't happen)
  return 'cloud';
}

/**
 * Filter an array of unified tracks by string
 */
export function filterUnifiedTracks(tracks: UnifiedTrack[], search: string): UnifiedTrack[] {
  if (search.length === 0) return tracks;

  return tracks.filter(
    (track) =>
      (track.artists && stripAccents(track.artists.toString().toLowerCase()).includes(search)) ||
      stripAccents(track.album.toLowerCase()).includes(search) ||
      (track.genres && stripAccents(track.genres.toString().toLowerCase()).includes(search)) ||
      stripAccents(track.title.toLowerCase()).includes(search),
  );
}

/**
 * Sort an array of unified tracks
 */
export function sortUnifiedTracks(
  tracks: UnifiedTrack[],
  sortBy: UnifiedSortConfig,
  sortOrder: SortOrder,
): UnifiedTrack[] {
  const firstOrder = sortOrder === 'Asc' ? 'asc' : 'desc';
  return orderBy<UnifiedTrack>(tracks, sortBy, [firstOrder]);
}

/**
 * Get status for unified tracks with file existence check
 */
export async function getUnifiedStatusWithFileCheck(tracks: UnifiedTrack[]): Promise<string> {
  const checkedTracks = await Promise.all(
    tracks.map(async (track) => ({
      ...track,
      location_type: await getLocationTypeWithFileCheck(track)
    }))
  );
  
  const duration = parseDuration(
    checkedTracks.map((d) => d.duration).reduce((a, b) => a + b, 0),
  );
  return `${checkedTracks.length} ${plural('track', checkedTracks.length)}, ${duration}`;
}

// Sort utilities for UnifiedTracks
const ARTIST = (t: UnifiedTrack): string =>
  t.artists ? stripAccents(t.artists.toString().toLowerCase()) : '';
const GENRE = (t: UnifiedTrack): string =>
  t.genres ? stripAccents(t.genres.toString().toLowerCase()) : '';
const ALBUM = (t: UnifiedTrack): string => 
  stripAccents(t.album.toLowerCase());
const TITLE = (t: UnifiedTrack): string => 
  stripAccents(t.title.toLowerCase());

type IterateeFunction = (track: UnifiedTrack) => string;
export type UnifiedSortConfig = Array<keyof UnifiedTrack | IterateeFunction>;

// Sort configurations
const UNIFIED_SORT_ORDERS: Record<SortBy, UnifiedSortConfig> = {
  Artist: [ARTIST, ALBUM, 'track_no'],
  Title: [TITLE, ARTIST, ALBUM, 'track_no'],
  Duration: ['duration', ARTIST, ALBUM, 'track_no'],
  Album: [ALBUM, ARTIST, 'track_no'],
  Genre: [GENRE, ARTIST, ALBUM, 'track_no'],
};

export function getUnifiedSortOrder(sortBy: SortBy): UnifiedSortConfig {
  return UNIFIED_SORT_ORDERS[sortBy];
}
