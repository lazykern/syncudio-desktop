import { invoke } from '@tauri-apps/api/core';

export interface LastFmSession {
    username: string;
    session_key: string;
}

/**
 * Last.fm scrobbler client for Syncudio
 */
export const lastfm = {
    async authenticate(username: string, password: string): Promise<string> {
        return invoke('plugin:lastfm|authenticate', { username, password });
    },

    async logout(): Promise<void> {
        return invoke('plugin:lastfm|logout');
    },

    async getSession(): Promise<LastFmSession | null> {
        return invoke('plugin:lastfm|get_session');
    },

    async scrobbleTrack(artist: string, title: string, album?: string): Promise<void> {
        return invoke('plugin:lastfm|scrobble_track', {
            artist,
            title,
            album,
        });
    },

    async updateNowPlaying(artist: string, title: string, album?: string): Promise<void> {
        return invoke('plugin:lastfm|update_now_playing', {
            artist,
            title,
            album,
        });
    },
};
