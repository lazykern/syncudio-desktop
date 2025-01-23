import orderBy from 'lodash-es/orderBy';
import type { UnifiedTrack, SortBy, SortOrder } from '../generated/typings';
import { stripAccents } from './utils-library';
import { parseDuration } from '../hooks/useFormattedDuration';
import { plural } from './localization';

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
 * Get status for unified tracks
 */
export function getUnifiedStatus(tracks: UnifiedTrack[]): string {
  const duration = parseDuration(
    tracks.map((d) => d.duration).reduce((a, b) => a + b, 0),
  );
  return `${tracks.length} ${plural('track', tracks.length)}, ${duration}`;
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
