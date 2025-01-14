import { useQuery } from '@tanstack/react-query';
import { useCallback, Suspense } from 'react';
import {
  Link,
  type LoaderFunctionArgs,
  redirect,
  useLoaderData,
  useParams,
} from 'react-router';

import TracksList from '../components/TracksList';
import * as ViewMessage from '../elements/ViewMessage';
import type { Track, Playlist } from '../generated/typings';
import useFilteredTracks from '../hooks/useFilteredTracks';
import useInvalidate from '../hooks/useInvalidate';
import usePlayingTrackID from '../hooks/usePlayingTrackID';
import config from '../lib/config';
import database from '../lib/database';
import PlaylistsAPI from '../stores/PlaylistsAPI';
import useLibraryStore from '../stores/useLibraryStore';
import type { LoaderData } from '../types/syncudio';

// Skeleton loading component
function PlaylistSkeleton() {
  return (
    <ViewMessage.Notice>
      <p>Loading playlist...</p>
    </ViewMessage.Notice>
  );
}

interface PlaylistContentProps {
  playlists: Playlist[];
  tracksDensity: 'compact' | 'normal';
}

function PlaylistContent({ playlists, tracksDensity }: PlaylistContentProps) {
  const { playlistID } = useParams();
  const trackPlayingID = usePlayingTrackID();
  const invalidate = useInvalidate();
  const search = useLibraryStore((state) => state.search);

  const { data: playlist } = useQuery({
    queryKey: ['playlist', playlistID],
    queryFn: async () => {
      if (!playlistID) throw new Error('No playlist ID');
      return database.getPlaylist(playlistID);
    },
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    enabled: !!playlistID
  });

  const { data: playlistTracks } = useQuery({
    queryKey: ['playlist-tracks', playlistID],
    queryFn: () => database.getTracks(playlist?.tracks || []),
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    enabled: !!playlist
  });

  const filteredTracks = useFilteredTracks(playlistTracks ?? []);

  const onReorder = useCallback(
    async (tracks: Track[]) => {
      if (playlistID != null) {
        await PlaylistsAPI.reorderTracks(playlistID, tracks);
        invalidate();
      }
    },
    [invalidate, playlistID],
  );

  if (!playlistTracks || playlistTracks.length === 0) {
    return (
      <ViewMessage.Notice>
        <p>Empty playlist</p>
        <ViewMessage.Sub>
          You can add tracks from the{' '}
          <Link to="/library" draggable={false}>
            library view
          </Link>
        </ViewMessage.Sub>
      </ViewMessage.Notice>
    );
  }

  if (filteredTracks.length === 0 && search.length > 0) {
    return (
      <ViewMessage.Notice>
        <p>Your search returned no results</p>
      </ViewMessage.Notice>
    );
  }

  return (
    <TracksList
      type="playlist"
      tracks={filteredTracks}
      tracksDensity={tracksDensity}
      trackPlayingID={trackPlayingID}
      playlists={playlists}
      onReorder={onReorder}
    />
  );
}

export default function ViewPlaylistDetails() {
  const { playlists, tracksDensity } = useLoaderData() as PlaylistLoaderData;

  return (
    <Suspense fallback={<PlaylistSkeleton />}>
      <PlaylistContent 
        playlists={playlists}
        tracksDensity={tracksDensity}
      />
    </Suspense>
  );
}

export type PlaylistLoaderData = LoaderData<typeof ViewPlaylistDetails.loader>;

ViewPlaylistDetails.loader = async ({ params }: LoaderFunctionArgs) => {
  if (typeof params.playlistID !== 'string') {
    throw new Error('Playlist ID is not defined');
  }

  try {
    const density = await config.get('track_view_density') as 'compact' | 'normal';
    if (density !== 'compact' && density !== 'normal') {
      throw new Error('Invalid track density value');
    }

    return {
      playlists: await database.getAllPlaylists(),
      tracksDensity: density,
    };
  } catch (err) {
    if (err === 'Playlist not found') {
      return redirect('/playlists');
    }
    throw err;
  }
};
