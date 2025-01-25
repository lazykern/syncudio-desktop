import { useCallback, useState, useEffect } from 'react';
import { useLoaderData } from 'react-router';

import * as Setting from '../components/Setting';
import CheckboxSetting from '../components/SettingCheckbox';
import Button from '../elements/Button';
import Flexbox from '../elements/Flexbox';
import { lastfm } from '../lib/lastfm';
import config from '../lib/config';
import { useToastsAPI } from '../stores/useToastsStore';
import useInvalidate, { useInvalidateCallback } from '../hooks/useInvalidate';
import type { SettingsLoaderData } from './settings';
import type { LastFmSession } from '../lib/lastfm';

export default function ViewSettingsScrobbler() {
  const { config: settings } = useLoaderData() as SettingsLoaderData;
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [authenticating, setAuthenticating] = useState(false);
  const [session, setSession] = useState<LastFmSession | null>(null);
  const toasts = useToastsAPI();
  const invalidate = useInvalidate();

  useEffect(() => {
    lastfm.getSession().then(setSession);
  }, []);

  const handleAuthenticate = useCallback(async () => {
    if (!username || !password) {
      toasts.add('warning', 'Please enter username and password');
      return;
    }

    setAuthenticating(true);
    try {
      await lastfm.authenticate(username, password);
      await config.set('lastfm_enabled', true);
      toasts.add('success', 'Successfully authenticated with Last.fm');
      setPassword(''); // Clear sensitive data
      const newSession = await lastfm.getSession();
      setSession(newSession);
      await invalidate();
    } catch (error) {
      toasts.add('danger', `Failed to authenticate: ${error}`);
    } finally {
      setAuthenticating(false);
    }
  }, [username, password, toasts, invalidate]);

  const handleLogout = useCallback(async () => {
    try {
      await lastfm.logout();
      await config.set('lastfm_enabled', false);
      toasts.add('success', 'Logged out from Last.fm');
      setSession(null);
      await invalidate();
    } catch (error) {
      toasts.add('danger', `Failed to logout: ${error}`);
    }
  }, [toasts, invalidate]);

  const handleToggleScrobbling = useCallback(async (value: boolean) => {
    await config.set('lastfm_enabled', value);
    await invalidate();
  }, [invalidate]);

  return (
    <div className="setting setting-scrobbler">
      <Setting.Section>
        <Setting.Title>Last.fm Settings</Setting.Title>
        <Setting.Description>
          Connect your Last.fm account to scrobble tracks as you listen.
        </Setting.Description>

        <CheckboxSetting
          slug="lastfm-enabled"
          title="Enable Scrobbling"
          description="Send your listening history to Last.fm"
          value={settings.lastfm_enabled}
          onChange={handleToggleScrobbling}
        />

        {!settings.lastfm_enabled && (
          <Setting.Description>
            Enable scrobbling to configure your Last.fm account.
          </Setting.Description>
        )}

        {settings.lastfm_enabled && !session && (
          <>
            <Setting.Input
              label="Last.fm Username"
              description="Your Last.fm account username"
              id="setting-lastfm-username"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              type="text"
            />

            <Setting.Input
              label="Last.fm Password"
              description="Your Last.fm account password"
              id="setting-lastfm-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              type="password"
            />

            <div style={{ marginTop: '1rem' }}>
              <Flexbox gap={4}>
                <Button
                  onClick={handleAuthenticate}
                  disabled={authenticating}
                >
                  {authenticating ? 'Authenticating...' : 'Login'}
                </Button>
              </Flexbox>
            </div>
          </>
        )}

        {settings.lastfm_enabled && session && (
          <>
            <Setting.Description>
              ✓ Connected to Last.fm as <strong>{session.username}</strong>. Your listening history will be scrobbled automatically.
            </Setting.Description>
            <div style={{ marginTop: '1rem' }}>
              <Flexbox gap={4}>
                <Button
                  onClick={handleLogout}
                  relevancy="danger"
                >
                  Logout
                </Button>
              </Flexbox>
            </div>
          </>
        )}
      </Setting.Section>
    </div>
  );
}
