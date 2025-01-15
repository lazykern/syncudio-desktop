import { useLoaderData } from 'react-router';
import type { SettingsLoaderData } from './settings';

import * as Setting from '../components/Setting';
import Flexbox from '../elements/Flexbox';
import Button from '../elements/Button';
import { useCallback, useEffect, useState } from 'react';
import { cloud } from '../lib/cloud-provider';
import { open } from '@tauri-apps/plugin-shell';

export default function SettingsCloud() {
  const { config } = useLoaderData() as SettingsLoaderData;
  const [isAuthorized, setIsAuthorized] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);

  useEffect(() => {
    // Check initial authorization status
    cloud.dropboxIsAuthorized().then(setIsAuthorized);
  }, []);

  const handleConnect = useCallback(async () => {
    try {
      setIsConnecting(true);
      // Get authorization URL
      const authUrl = await cloud.dropboxStartAuthorization();
      
      // Open auth URL in default browser
      await open(authUrl);

      // Show instructions to user
      alert('Please complete the authorization in your browser and paste the authorization code here.');
      
      // Prompt for auth code
      const authCode = prompt('Please paste the authorization code:');
      if (!authCode) {
        throw new Error('Authorization cancelled');
      }

      // Complete authorization with code
      await cloud.dropboxCompleteAuthorization(authCode);
      setIsAuthorized(true);
    } catch (error) {
      console.error('Failed to connect to Dropbox:', error);
      alert('Failed to connect to Dropbox. Please try again.');
    } finally {
      setIsConnecting(false);
    }
  }, []);

  const handleDisconnect = useCallback(async () => {
    try {
      await cloud.dropboxUnauthorize();
      setIsAuthorized(false);
    } catch (error) {
      console.error('Failed to disconnect from Dropbox:', error);
      alert('Failed to disconnect from Dropbox. Please try again.');
    }
  }, []);

  return (
    <div className="setting setting-cloud">
      <Setting.Section>
        <Setting.Title>Dropbox</Setting.Title>
        <Setting.Description>
          Connect your Dropbox account to sync your music library across devices.
        </Setting.Description>
        <Flexbox>
          {!isAuthorized ? (
            <Button
              onClick={handleConnect}
              disabled={isConnecting}
            >
              {isConnecting ? 'Connecting...' : 'Connect to Dropbox'}
            </Button>
          ) : (
            <Button
              onClick={handleDisconnect}
              relevancy="danger"
            >
              Disconnect from Dropbox
            </Button>
          )}
        </Flexbox>
      </Setting.Section>
    </div>
  );
}
