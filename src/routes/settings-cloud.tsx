import { useCallback, useEffect, useState } from 'react';
import { useLoaderData } from 'react-router';
import { open } from '@tauri-apps/plugin-shell';

import * as Setting from '../components/Setting';
import type { SettingsLoaderData } from './settings';
import Flexbox from '../elements/Flexbox';
import Button from '../elements/Button';
import { cloud } from '../lib/cloud';

export default function ViewSettingsSync() {
  const { config } = useLoaderData() as SettingsLoaderData;
  const [isDropboxConnected, setIsDropboxConnected] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const [showAuthInput, setShowAuthInput] = useState(false);
  const [authCode, setAuthCode] = useState('');

  useEffect(() => {
    // Check initial Dropbox connection status
    cloud.dropboxIsAuthorized().then(setIsDropboxConnected);
  }, []);

  const handleDropboxConnect = useCallback(async () => {
    try {
      setIsConnecting(true);
      // Get the authorization URL
      const authUrl = await cloud.dropboxStartAuthorization();

      // Open the auth URL in the default browser
      await open(authUrl);

      // Show the auth code input
      setShowAuthInput(true);
    } catch (error) {
      console.error('Failed to start Dropbox authorization:', error);
      setIsConnecting(false);
    }
  }, []);

  const handleAuthCodeSubmit = useCallback(async () => {
    if (!authCode.trim()) return;

    try {
      await cloud.dropboxCompleteAuthorization(authCode.trim());
      setIsDropboxConnected(true);
      setShowAuthInput(false);
      setAuthCode('');
    } catch (error) {
      console.error('Failed to complete Dropbox authorization:', error);
    } finally {
      setIsConnecting(false);
    }
  }, [authCode]);

  const handleDropboxDisconnect = useCallback(async () => {
    try {
      await cloud.dropboxUnauthorize();
      setIsDropboxConnected(false);
    } catch (error) {
      console.error('Failed to disconnect Dropbox:', error);
    }
  }, []);

  return (
    <div className="setting setting-sync">
      <Setting.Section>
        <Setting.Title>Dropbox</Setting.Title>
        <Flexbox>
        {showAuthInput ? (
          <Flexbox direction="vertical" gap={8}>
            <p>
              Please copy the authorization code from the browser and paste it
              here:
            </p>
            <Setting.Input
              label="Authorization Code"
              id="dropbox-auth-code"
              type="text"
              value={authCode}
              onChange={(e) => setAuthCode(e.currentTarget.value)}
              placeholder="Enter authorization code"
            />
            <Flexbox gap={8}>
              <Button
                onClick={handleAuthCodeSubmit}
                disabled={!authCode.trim()}
              >
                Submit
              </Button>
              <Button
                onClick={() => {
                  setShowAuthInput(false);
                  setIsConnecting(false);
                  setAuthCode('');
                }}
              >
                Cancel
              </Button>
            </Flexbox>
          </Flexbox>
        ) : isDropboxConnected ? (
          <Button onClick={handleDropboxDisconnect}>Disconnect</Button>
        ) : (
          <Button onClick={handleDropboxConnect} disabled={isConnecting}>
            {isConnecting ? 'Connecting...' : 'Connect'}
          </Button>
        )}
        </Flexbox>
      </Setting.Section>
      <Setting.Section>
        <Setting.Title>Google Drive</Setting.Title>
        <p>Google Drive integration coming soon.</p>
        <Flexbox>
          <Button disabled>Connect</Button>
        </Flexbox>
      </Setting.Section>
    </div>
  );
}
