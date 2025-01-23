import { useQuery } from '@tanstack/react-query';
import { useMemo } from 'react';
import { Link, useLoaderData } from 'react-router';

import UnifiedTracksList from '../components/UnifiedTracksList';
import View from '../elements/View';
import * as ViewMessage from '../elements/ViewMessage';
import useFilteredUnifiedTracks from '../hooks/useFilteredUnifiedTracks';
import usePlayingTrackID from '../hooks/usePlayingTrackID';
import config from '../lib/config';
import database from '../lib/database';
import { cloudDatabase } from '../lib/cloud-database';
import useLibraryStore from '../stores/useLibraryStore';
import type { LoaderData } from '../types/syncudio';
import type { UnifiedTrack } from '../generated/typings';

export default function ViewLibrary() {
  const trackPlayingID = usePlayingTrackID();
  const refreshing = useLibraryStore((state) => state.refreshing);
  const search = useLibraryStore((state) => state.search);
  const sortBy = useLibraryStore((state) => state.sortBy);
  const sortOrder = useLibraryStore((state) => state.sortOrder);

  const { playlists, tracksDensity } = useLoaderData() as LibraryLoaderData;

  // Get unified tracks that include both local and cloud tracks
  const { data: tracks, isLoading } = useQuery({
    queryKey: ['unified-tracks'],
    queryFn: cloudDatabase.getUnifiedTracks,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  });

  const filteredTracks = useFilteredUnifiedTracks(tracks ?? [], sortBy, sortOrder);

  const getLibraryComponent = useMemo(() => {
    // Refreshing library
    if (isLoading) {
      return (
        <ViewMessage.Notice>
          <p>Loading library...</p>
        </ViewMessage.Notice>
      );
    }

    // Refreshing library
    if (refreshing) {
      return (
        <ViewMessage.Notice>
          <p>Your library is being scanned =)</p>
          <ViewMessage.Sub>hold on...</ViewMessage.Sub>
        </ViewMessage.Notice>
      );
    }

    // Empty library
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

    // Empty search
    if (filteredTracks.length === 0) {
      return (
        <ViewMessage.Notice>
          <p>Your search returned no results</p>
        </ViewMessage.Notice>
      );
    }

    // All good !
    return (
      <UnifiedTracksList
        type="library"
        tracks={filteredTracks}
        tracksDensity={tracksDensity}
        trackPlayingID={trackPlayingID}
        playlists={playlists}
      />
    );
  }, [
    search,
    refreshing,
    filteredTracks,
    playlists,
    trackPlayingID,
    tracksDensity,
    isLoading,
  ]);

  return <View>{getLibraryComponent}</View>;
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
