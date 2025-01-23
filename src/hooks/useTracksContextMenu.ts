import { useCallback } from 'react';
import { Menu, MenuItem, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import { useNavigate } from 'react-router';
import type { Track, Playlist } from '../generated/typings';
import { logAndNotifyError } from '../lib/utils';
import PlaylistsAPI from '../stores/PlaylistsAPI';
import { usePlayerAPI } from '../stores/usePlayerStore';
import { useLibraryAPI } from '../stores/useLibraryStore';

interface UseTracksContextMenuProps {
  tracks: Track[];
  selectedTracks: Set<string>;
  type: string;
  playlists: Playlist[];
  currentPlaylist?: string;
  playerAPI: ReturnType<typeof usePlayerAPI>;
  libraryAPI: ReturnType<typeof useLibraryAPI>;
  invalidate: () => void;
}

interface UseTracksContextMenuReturn {
  showContextMenu: (e: React.MouseEvent, index: number) => Promise<void>;
}

export function useTracksContextMenu({
  tracks,
  selectedTracks,
  type,
  playlists,
  currentPlaylist,
  playerAPI,
  libraryAPI,
  invalidate,
}: UseTracksContextMenuProps): UseTracksContextMenuReturn {
  const navigate = useNavigate();

  const showContextMenu = useCallback(
    async (e: React.MouseEvent, index: number) => {
      e.preventDefault();

      const selectedCount = selectedTracks.size;
      const track = tracks[index];
      let shownPlaylists = playlists;

      // Hide current playlist if one the given playlist view
      if (type === 'playlist') {
        shownPlaylists = playlists.filter(
          (elem) => elem.id !== currentPlaylist,
        );
      }

      // Playlist sub-menu
      const playlistSubMenu = await Promise.all([
        MenuItem.new({
          text: 'Create new playlist...',
          async action() {
            await PlaylistsAPI.create(
              'New playlist',
              Array.from(selectedTracks),
            );
            invalidate();
          },
        }),
        PredefinedMenuItem.new({
          item: 'Separator',
        }),
      ]);

      if (shownPlaylists.length === 0) {
        playlistSubMenu.push(
          await MenuItem.new({ text: 'No playlists', enabled: false }),
        );
      } else {
        playlistSubMenu.push(
          ...(await Promise.all(
            shownPlaylists.map((playlist) =>
              MenuItem.new({
                text: playlist.name,
                async action() {
                  await PlaylistsAPI.addTracks(
                    playlist.id,
                    Array.from(selectedTracks),
                  );
                },
              }),
            ),
          )),
        );
      }

      const menuItems = await Promise.all([
        MenuItem.new({
          text:
            selectedCount > 1
              ? `${selectedCount} tracks selected`
              : `${selectedCount} track selected`,
          enabled: false,
        }),
        PredefinedMenuItem.new({
          text: '?',
          item: 'Separator',
        }),
        MenuItem.new({
          text: 'Add to queue',
          action() {
            playerAPI.addInQueue(Array.from(selectedTracks));
          },
        }),
        MenuItem.new({
          text: 'Play next',
          action() {
            playerAPI.addNextInQueue(Array.from(selectedTracks));
          },
        }),
        PredefinedMenuItem.new({
          item: 'Separator',
        }),
        Submenu.new({
          text: 'Add to playlist',
          items: playlistSubMenu,
        }),
        PredefinedMenuItem.new({
          text: '?',
          item: 'Separator',
        }),
      ]);

      menuItems.push(
        ...(await Promise.all(
          track.artists.map((artist) =>
            MenuItem.new({
              text: `Search for "${artist}" `,
              action: () => {
                libraryAPI.search(artist);
              },
            }),
          ),
        )),
      );

      menuItems.push(
        await MenuItem.new({
          text: `Search for "${track.album}"`,
          action() {
            libraryAPI.search(track.album);
          },
        }),
      );

      if (type === 'playlist' && currentPlaylist) {
        menuItems.push(
          ...(await Promise.all([
            PredefinedMenuItem.new({ item: 'Separator' }),
            MenuItem.new({
              text: 'Remove from playlist',
              async action() {
                await PlaylistsAPI.removeTracks(
                  currentPlaylist,
                  Array.from(selectedTracks),
                );
                invalidate();
              },
            }),
          ])),
        );
      }

      menuItems.push(
        ...(await Promise.all([
          PredefinedMenuItem.new({ item: 'Separator' }),
          MenuItem.new({
            text: 'Edit track',
            action: () => {
              navigate(`/details/${track.id}`);
            },
          }),
          PredefinedMenuItem.new({ item: 'Separator' }),
          MenuItem.new({
            text: 'Show in file manager',
            action: async () => {
              await revealItemInDir(track.path);
            },
          }),
          MenuItem.new({
            text: 'Remove from library',
            action: async () => {
              await libraryAPI.remove(Array.from(selectedTracks));
              invalidate();
            },
          }),
        ])),
      );

      const menu = await Menu.new({
        items: menuItems,
      });

      await menu.popup().catch(logAndNotifyError);
    },
    [
      currentPlaylist,
      playlists,
      selectedTracks,
      tracks,
      type,
      navigate,
      playerAPI,
      libraryAPI,
      invalidate,
    ],
  );

  return { showContextMenu };
} 