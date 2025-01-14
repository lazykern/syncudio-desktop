// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

export type CloudFile = { id: string, name: string, parent_id: string | null, size: bigint, is_folder: boolean, modified_at: bigint, created_at: bigint, mime_type: string | null, hash: FileHash | null, };

export type CloudFolder = { id: string, provider_type: string, cloud_folder_id: string, cloud_folder_name: string, local_folder_path: string, created_at: bigint, updated_at: bigint, };

export type CloudSync = { id: string, provider_type: string, folder_id: string, item_id: string, item_type: string, cloud_file_id: string, cloud_file_name: string, local_path: string, last_synced: bigint | null, sync_status: string, created_at: bigint, updated_at: bigint, };

export type Config = { theme: string, audio_volume: number, audio_playback_rate: number | null, audio_output_device: string, audio_muted: boolean, audio_shuffle: boolean, audio_repeat: Repeat, default_view: DefaultView, library_sort_by: SortBy, library_sort_order: SortOrder, library_autorefresh: boolean, sleepblocker: boolean, auto_update_checker: boolean, minimize_to_tray: boolean, notifications: boolean, track_view_density: string, };

export type DefaultView = "Library" | "Playlists";

export type FileHash = { "Sha1": string } | { "Sha256": string } | { "ContentHash": string };

export type IPCEvent = { "Unknown": string } | "PlaybackPlay" | "PlaybackPause" | "PlaybackStop" | "PlaybackPlayPause" | "PlaybackPrevious" | "PlaybackNext" | "PlaybackStart" | "LibraryScanProgress" | "GoToLibrary" | "GoToPlaylists" | "GoToSettings" | "JumpToPlayingTrack";

export type LocalFolder = { path: string, };

/** ----------------------------------------------------------------------------
 * Playlist
 * represent a playlist, that has a name and a list of tracks
 * -------------------------------------------------------------------------- */
export type Playlist = { id: string, name: string, tracks: Array<string>, import_path: string | null, };

export type Repeat = "All" | "One" | "None";

/**
 * Scan progress information
 */
export type ScanProgress = { current: number, total: number, };

/**
 * Result of a library scan operation
 */
export type ScanResult = { track_count: number, track_failures: number, playlist_count: number, playlist_failures: number, };

export type SortBy = "Artist" | "Album" | "Title" | "Duration" | "Genre";

export type SortOrder = "Asc" | "Dsc";

/**
 * Track
 * represent a single track
 */
export type Track = { id: string, local_folder_path: string, relative_path: string, title: string, album: string, artists: Array<string>, genres: Array<string>, year: number | null, duration: number, track_no: number | null, track_of: number | null, disk_no: number | null, disk_of: number | null, index_hash: string | null, };
