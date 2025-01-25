import { getCurrentWindow } from '@tauri-apps/api/window';
import { sendNotification } from '@tauri-apps/plugin-notification';
import { useEffect } from 'react';

import config from '../lib/config';
import { getCover } from '../lib/cover';
import player from '../lib/player';
import { lastfm } from '../lib/lastfm';
import { logAndNotifyError } from '../lib/utils';
import { usePlayerAPI } from '../stores/usePlayerStore';
import { useToastsAPI } from '../stores/useToastsStore';

const SCROBBLE_THRESHOLD = 0.5; // 50%

const AUDIO_ERRORS = {
  aborted: 'The video playback was aborted.',
  corrupt: 'The audio playback was aborted due to a corruption problem.',
  notFound:
    'The track file could not be found. It may be due to a file move or an unmounted partition.',
  unknown: 'An unknown error occurred.',
};

/**
 * Handle some of the logic regarding the player. Technically, it should not be there,
 * but part of the Player library, but cleaning up events in case of hot-reload is tough
 */
function PlayerEvents() {
  const playerAPI = usePlayerAPI();
  const toastsAPI = useToastsAPI();

  useEffect(() => {
    function handleAudioError(e: ErrorEvent) {
      playerAPI.stop();

      const element = e.target as HTMLAudioElement;

      if (element) {
        const { error } = element;

        if (!error) return;

        switch (error.code) {
          case error.MEDIA_ERR_ABORTED:
            toastsAPI.add('warning', AUDIO_ERRORS.aborted);
            break;
          case error.MEDIA_ERR_DECODE:
            toastsAPI.add('danger', AUDIO_ERRORS.corrupt);
            break;
          case error.MEDIA_ERR_SRC_NOT_SUPPORTED:
            toastsAPI.add('danger', AUDIO_ERRORS.notFound);
            break;
          default:
            toastsAPI.add('danger', AUDIO_ERRORS.unknown);
            break;
        }
      }
    }

async function handleTrackChange() {
  const track = player.getTrack();
  const notificationsEnabled = await config.get('notifications');
  const isMinimized = await getCurrentWindow()
    .isMinimized()
    .catch(logAndNotifyError);
  const isFocused = await getCurrentWindow()
    .isFocused()
    .catch(logAndNotifyError);

  // Handle notifications
  if (track && notificationsEnabled && !isFocused && isMinimized) {
    const cover = await getCover(track.path);
    sendNotification({
      title: track.title,
      body: `${track.artists.join(', ')}\n${track.album}`,
      silent: true,
      icon: cover ?? undefined,
    });
  }

  // Update Last.fm now playing if enabled and track exists
  if (track) {
    const lastfmEnabled = await config.get('lastfm_enabled');
    if (lastfmEnabled) {
      try {
        await lastfm.updateNowPlaying(
          track.artists.join(', '),
          track.title,
          track.album
        );
      } catch (error) {
        console.warn('Failed to update Last.fm now playing:', error);
      }
    }
  }
}

    // Track play duration for scrobbling
let hasScrobbled = false;

async function handleTimeUpdate() {
  const track = player.getTrack();
  const audio = player.getAudio();
  
  // Only proceed if we have a track playing and haven't scrobbled yet
  if (!track || hasScrobbled || audio.paused) return;

  const playedPercentage = audio.currentTime / audio.duration;
  if (playedPercentage >= SCROBBLE_THRESHOLD) {
    // Check Last.fm enabled state before attempting to scrobble
    const lastfmEnabled = await config.get('lastfm_enabled');
    if (lastfmEnabled) {
      hasScrobbled = true;
      try {
        await lastfm.scrobbleTrack(
          track.artists.join(', '),
          track.title,
          track.album
        );
      } catch (error) {
        console.warn('Failed to scrobble track:', error);
        // Reset scrobble flag on error to allow retry
        hasScrobbled = false;
      }
    }
  }
}

    // Reset scrobble flag on track change
    function handlePlay() {
      hasScrobbled = false;
      handleTrackChange();
    }

    // Bind player events
    const audio = player.getAudio();
    audio.addEventListener('play', handlePlay);
    audio.addEventListener('error', handleAudioError);
    audio.addEventListener('ended', playerAPI.next);
    audio.addEventListener('timeupdate', handleTimeUpdate);

    return function cleanup() {
      audio.removeEventListener('play', handlePlay);
      audio.removeEventListener('error', handleAudioError);
      audio.removeEventListener('ended', playerAPI.next);
      audio.removeEventListener('timeupdate', handleTimeUpdate);
    };
  }, [toastsAPI, playerAPI]);

  return null;
}

export default PlayerEvents;
