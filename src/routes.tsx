import { type RouteObject, createHashRouter } from 'react-router';

import GlobalErrorBoundary from './components/GlobalErrorBoundary';
import RootView from './routes/Root';
import ViewLibrary from './routes/library';
import ViewPlaylistDetails from './routes/playlist-details';
import ViewPlaylists from './routes/playlists';
import ViewSettings from './routes/settings';
import ViewSettingsAbout from './routes/settings-about';
import ViewSettingsAudio from './routes/settings-audio';
import ViewSettingsLibrary from './routes/settings-library';
import ViewSettingsCloud from './routes/settings-cloud';
import ViewSettingsUI from './routes/settings-ui';
import ViewSettingsScrobbler from './routes/settings-scrobbler';
import ViewTrackDetails from './routes/track-details';
import ViewCloud from './routes/cloud';

const routeTree: RouteObject[] = [
  {
    path: '/',
    id: 'root',
    Component: RootView,
    loader: RootView.loader,
    HydrateFallback: () => null, // there should be no hydration as we're SPA-only
    ErrorBoundary: GlobalErrorBoundary,
    children: [
      {
        path: 'library',
        id: 'library',
        Component: ViewLibrary,
        loader: ViewLibrary.loader,
      },
      {
        path: 'playlists',
        id: 'playlists',
        Component: ViewPlaylists,
        loader: ViewPlaylists.loader,
        children: [
          {
            path: ':playlistID',
            id: 'playlist-details',
            Component: ViewPlaylistDetails,
            loader: ViewPlaylistDetails.loader,
          },
        ],
      },
      {
        path: 'cloud',
        id: 'cloud',
        Component: ViewCloud,
      },
      {
        path: 'settings',
        id: 'settings',
        Component: ViewSettings,
        children: [
          {
            path: 'library',
            Component: ViewSettingsLibrary,
            loader: ViewSettings.loader,
          },
          {
            path: 'interface',
            Component: ViewSettingsUI,
            loader: ViewSettings.loader,
          },
          {
            path: 'audio',
            Component: ViewSettingsAudio,
            loader: ViewSettings.loader,
          },
          {
            path: 'cloud',
            Component: ViewSettingsCloud,
            loader: ViewSettings.loader,
          },
          {
            path: 'scrobbler',
            Component: ViewSettingsScrobbler,
            loader: ViewSettings.loader,
          },
          {
            path: 'about',
            Component: ViewSettingsAbout,
            loader: ViewSettings.loader,
          },
        ],
      },
      {
        path: 'details/:trackID',
        Component: ViewTrackDetails,
        loader: ViewTrackDetails.loader,
      },
    ],
  },
];

const router = createHashRouter(routeTree);

export default router;
