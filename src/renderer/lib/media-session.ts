import usePlayerStore from '../stores/usePlayerStore';

import Player from './player';

/**
 * Integration for MediaSession (mpris, macOS player controls etc)...
 */
export default function initMediaSession(player: Player) {
  player.getAudio().addEventListener('loadstart', async () => {
    const track = player.getTrack();
    if (track) {
      const cover = await window.MuseeksAPI.covers.getCoverAsBase64(track);

      navigator.mediaSession.metadata = new MediaMetadata({
        title: track.title,
        artist: track.artist.join(', '),
        album: track.album,
        artwork: cover ? [{ src: cover }] : undefined,
      });
    }
  });

  player.getAudio().addEventListener('play', async () => {
    navigator.mediaSession.playbackState = 'playing';
  });

  player.getAudio().addEventListener('pause', async () => {
    navigator.mediaSession.playbackState = 'paused';
  });

  const playerAPI = usePlayerStore.getState().api;

  navigator.mediaSession.setActionHandler('play', async () => {
    playerAPI.play();
  });

  navigator.mediaSession.setActionHandler('pause', async () => {
    playerAPI.pause();
  });

  navigator.mediaSession.setActionHandler('previoustrack', async () => {
    playerAPI.previous();
  });

  navigator.mediaSession.setActionHandler('nexttrack', async () => {
    playerAPI.next();
  });
}