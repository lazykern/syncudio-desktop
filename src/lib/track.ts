import type { Track } from '../generated/typings';

/**
 * Get the full path of a track by joining its local_folder_path and relative_path.
 * This is a synchronous operation to avoid the overhead of IPC calls.
 */
export function getTrackPath(track: Track): string {
  // Use platform-specific path separator
  const sep = navigator.platform.startsWith('Win') ? '\\' : '/';
  return `${track.local_folder_path}${sep}${track.relative_path}`;
} 