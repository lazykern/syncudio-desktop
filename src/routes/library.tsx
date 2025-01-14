import { useQuery } from '@tanstack/react-query';
import { useMemo, Suspense } from 'react';
import { Link, useLoaderData } from 'react-router';

import TracksList from '../components/TracksList';
import View from '../elements/View';
import * as ViewMessage from '../elements/ViewMessage';
import useFilteredTracks from '../hooks/useFilteredTracks';
import usePlayingTrackID from '../hooks/usePlayingTrackID';
import config from '../lib/config';
import database from '../lib/database';
import useLibraryStore from '../stores/useLibraryStore';
import type { LoaderData } from '../types/syncudio';
import type { Track, Playlist } from '../generated/typings';

// Skeleton loading component
function TracksListSkeleton() {
  return (
    <ViewMessage.Notice>
      <p>Loading tracks...</p>
    </ViewMessage.Notice>
  );
}

// Tracks list with data loading
interface TracksListWithDataProps {
  tracksDensity: 'compact' | 'normal';
  playlists: Playlist[];
}

function TracksListWithData({ tracksDensity, playlists }: TracksListWithDataProps) {
  const trackPlayingID = usePlayingTrackID();
  const refreshing = useLibraryStore((state) => state.refreshing);
  const search = useLibraryStore((state) => state.search);
  const sortBy = useLibraryStore((state) => state.sortBy);
  const sortOrder = useLibraryStore((state) => state.sortOrder);

  const { data: tracks } = useQuery({
    queryKey: ['tracks'],
    queryFn: database.getAllTracks,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false
  });

  const filteredTracks = useFilteredTracks(tracks ?? [], sortBy, sortOrder);

  if (refreshing) {
    return (
      <ViewMessage.Notice>
        <p>Your library is being scanned =)</p>
        <ViewMessage.Sub>hold on...</ViewMessage.Sub>
      </ViewMessage.Notice>
    );
  }

  if (filteredTracks.length === 0 && search === '') {
    return (
      <ViewMessage.Notice>
        <p>There is no music in your library :(</p>
        <ViewMessage.Sub>
          <span>you can</span>{' '}
          <Link to="/settings/library" draggable={false}>
            add your music here
          </Link>
        </ViewMessage.Sub>
      </ViewMessage.Notice>
    );
  }

  if (filteredTracks.length === 0) {
    return (
      <ViewMessage.Notice>
        <p>Your search returned no results</p>
      </ViewMessage.Notice>
    );
  }

  return (
    <TracksList
      type="library"
      tracks={filteredTracks}
      tracksDensity={tracksDensity}
      trackPlayingID={trackPlayingID}
      playlists={playlists}
    />
  );
}

export default function ViewLibrary() {
  const { playlists, tracksDensity } = useLoaderData() as LibraryLoaderData;

  return (
    <View>
      <Suspense fallback={<TracksListSkeleton />}>
        <TracksListWithData 
          tracksDensity={tracksDensity}
          playlists={playlists}
        />
      </Suspense>
    </View>
  );
}

export type LibraryLoaderData = LoaderData<typeof ViewLibrary.loader>;

ViewLibrary.loader = async () => {
  return {
    playlists: await database.getAllPlaylists(),
    tracksDensity: (await config.get('track_view_density')) as
      | 'compact'
      | 'normal',
  };
};

// ViewLibrary.whyDidYouRender = true;
