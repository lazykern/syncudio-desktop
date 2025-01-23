import type { Track, UnifiedTrack } from '../generated/typings';
import { parseDuration } from '../hooks/useFormattedDuration';
import { plural } from './localization';

type TrackStatusItem = {
  duration: number;
  artists: string[] | null;
  title: string;
  album: string;
};

export function getTrackStatus(tracks: (Track | UnifiedTrack)[]): string {
  const duration = parseDuration(
    tracks.map((d) => d.duration).reduce((a, b) => a + b, 0),
  );
  return `${tracks.length} ${plural('track', tracks.length)}, ${duration}`;
}

// Helper function to convert UnifiedTrack to Track-like structure for status
export function toTrackStatusItem(track: Track | UnifiedTrack): TrackStatusItem {
  return {
    duration: track.duration,
    artists: 'artists' in track ? track.artists : null,
    title: track.title,
    album: track.album,
  };
}
