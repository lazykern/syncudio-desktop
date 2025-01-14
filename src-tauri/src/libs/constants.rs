// Database-related constants

// KEEP THAT IN SYNC with Tauri's file associations in tauri.conf.json
pub const SUPPORTED_TRACKS_EXTENSIONS: [&str; 9] = [
    "mp3", "aac", "m4a", "3gp", "wav", /* mp3 / mp4 */
    "ogg", "opus", /* Opus */
    "flac", /* Flac */
    "weba", /* Web media */
];

pub const SUPPORTED_PLAYLISTS_EXTENSIONS: [&str; 1] = ["m3u"];

// Cloud provider types
pub const CLOUD_PROVIDER_DROPBOX: &str = "dropbox";
pub const CLOUD_PROVIDER_GDRIVE: &str = "gdrive"; 