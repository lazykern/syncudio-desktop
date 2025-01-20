import { useLoaderData } from 'react-router';
import type { SettingsLoaderData } from './settings';
import type { CloudFile, CloudFolder, CloudProviderType } from '../generated/typings';

import * as Setting from '../components/Setting';
import Flexbox from '../elements/Flexbox';
import Button from '../elements/Button';
import { useCallback, useEffect, useState } from 'react';
import { cloud } from '../lib/cloud-provider';
import { cloudDatabase } from '../lib/cloud-database';
import { open } from '@tauri-apps/plugin-shell';
import CloudFolderSelect from '../components/CloudFolderSelect';
import { open as openDialog, ask } from '@tauri-apps/plugin-dialog';
import { useToastsAPI } from '../stores/useToastsStore';
import styles from './settings-cloud.module.css';

// Helper function to handle BigInt serialization
function bigIntReplacer(_key: string, value: any) {
  return typeof value === 'bigint' ? value.toString() : value;
}

export default function SettingsCloud() {
  const { config } = useLoaderData() as SettingsLoaderData;
  const [isAuthorized, setIsAuthorized] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const [showCloudFolderSelect, setShowCloudFolderSelect] = useState(false);
  const [cloudFolders, setCloudFolders] = useState<CloudFolder[]>([]);
  const [authCode, setAuthCode] = useState('');
  const [showAuthInput, setShowAuthInput] = useState(false);
  const toastsAPI = useToastsAPI();

  useEffect(() => {
    // Check initial authorization status
    cloud.dropboxIsAuthorized().then(setIsAuthorized);
    // Load existing cloud folders
    if (isAuthorized) {
      loadCloudFolders();
    }
  }, [isAuthorized]);

  const loadCloudFolders = async () => {
    try {
      const folders = await cloudDatabase.getCloudFoldersByProvider('dropbox');
      setCloudFolders(folders);
    } catch (error) {
      console.error('Failed to load cloud folders:', error);
      toastsAPI.add('danger', 'Failed to load cloud folders. Please try again.');
    }
  };

  const handleConnect = useCallback(async () => {
    try {
      // Get authorization URL
      const authUrl = await cloud.dropboxStartAuthorization();
      
      // Open auth URL in default browser
      await open(authUrl);

      // Show auth code input
      setShowAuthInput(true);
    } catch (error) {
      console.error('Failed to connect to Dropbox:', error);
      toastsAPI.add('danger', 'Failed to connect to Dropbox. Please try again.');
    }
  }, [toastsAPI]);

  const handleAuthSubmit = useCallback(async () => {
    try {
      if (!authCode) {
        toastsAPI.add('danger', 'Please enter the authorization code');
        return;
      }

      setIsConnecting(true);

      await cloud.dropboxCompleteAuthorization(authCode);
      setIsAuthorized(true);
      setShowAuthInput(false);
      setAuthCode('');
      toastsAPI.add('success', 'Successfully connected to Dropbox!');
    } catch (error) {
      console.error('Failed to connect to Dropbox:', error);
      toastsAPI.add('danger', 'Failed to connect to Dropbox. Please try again.');
    } finally {
      setIsConnecting(false);
    }
  }, [authCode, toastsAPI]);

  const handleAuthCancel = useCallback(() => {
    setShowAuthInput(false);
    setAuthCode('');
    setIsConnecting(false);
  }, []);

  const handleDisconnect = useCallback(async () => {
    try {
      const confirmed = await ask('Are you sure you want to disconnect from Dropbox?', {
        title: 'Confirm Disconnect',
        kind: 'warning'
      });

      if (!confirmed) return;

      await cloud.dropboxUnauthorize();
      setIsAuthorized(false);
      setCloudFolders([]);
      toastsAPI.add('success', 'Successfully disconnected from Dropbox');
    } catch (error) {
      console.error('Failed to disconnect from Dropbox:', error);
      toastsAPI.add('danger', 'Failed to disconnect from Dropbox. Please try again.');
    }
  }, [toastsAPI]);

  const handleAddCloudFolder = useCallback(() => {
    setShowCloudFolderSelect(true);
  }, []);

  const handleCloudFolderSelect = useCallback(async (cloudFile: CloudFile, fullPath: string) => {
    try {
      // Open dialog to select local folder
      const localPath = await openDialog({
        directory: true,
        multiple: false,
        title: 'Select Local Folder'
      });

      if (!localPath || typeof localPath !== 'string') {
        return;
      }

      const timestamp = Date.now();
      // Create cloud folder mapping with BigInt serialization
      const folder: CloudFolder = {
        id: crypto.randomUUID(),
        provider_type: 'dropbox',
        cloud_folder_id: cloudFile.id,
        cloud_folder_path: fullPath,
        local_folder_path: localPath,
      };

      // Save to database with BigInt serialization
      await cloudDatabase.saveCloudFolder(JSON.parse(JSON.stringify(folder, bigIntReplacer)));
      
      // Refresh list
      await loadCloudFolders();
      
      // Close modal
      setShowCloudFolderSelect(false);
      toastsAPI.add('success', 'Cloud folder added successfully');
    } catch (error) {
      console.error('Failed to add cloud folder:', error);
      toastsAPI.add('danger', 'Failed to add cloud folder. Please try again.');
    }
  }, [toastsAPI]);

  const handleRemoveCloudFolder = useCallback(async (folderId: string) => {
    try {
      const confirmed = await ask('Are you sure you want to remove this cloud folder?', {
        title: 'Confirm Remove',
        kind: 'warning'
      });

      if (!confirmed) return;

      await cloudDatabase.deleteCloudFolder(folderId);
      await loadCloudFolders();
      toastsAPI.add('success', 'Cloud folder removed successfully');
    } catch (error) {
      console.error('Failed to remove cloud folder:', error);
      toastsAPI.add('danger', 'Failed to remove cloud folder. Please try again.');
    }
  }, [toastsAPI]);

  return (
    <div className="setting setting-cloud">
      <Setting.Section>
        <Setting.Title>Dropbox</Setting.Title>
        <Setting.Description>
          Connect your Dropbox account to sync your music library across devices.
        </Setting.Description>
        <Flexbox gap={4}>
          {!isAuthorized ? (
            <>
              {!showAuthInput ?
                <Button onClick={handleConnect}>
                  Connect to Dropbox
                </Button>
              : (
                <div className={styles.authInput}>
                  <Setting.Description>
                    Please complete the authorization in your browser, then paste the code below:
                  </Setting.Description>
                  <input
                    type="text"
                    value={authCode}
                    onChange={(e) => setAuthCode(e.target.value)}
                    placeholder="Paste authorization code here"
                    className={styles.input}
                  />
                  <Flexbox gap={4}>
                    <Button onClick={handleAuthSubmit} disabled={isConnecting}>Submit</Button>
                    <Button onClick={handleAuthCancel} relevancy="danger">Cancel</Button>
                  </Flexbox>
                </div>
              )}
            </>
          ) : (
            <>
              <Button onClick={handleAddCloudFolder}>
                Add Cloud Folder
              </Button>
              <Button
                onClick={handleDisconnect}
                relevancy="danger"
              >
                Disconnect from Dropbox
              </Button>
            </>
          )}
        </Flexbox>

        {cloudFolders.length > 0 && (
          <div className={styles.cloudFolders}>
            <h3>Sync Folders</h3>
            <ul>
              {cloudFolders.map(folder => (
                <li key={folder.id}>
                  <span>{folder.cloud_folder_path} → {folder.local_folder_path}</span>
                  <Flexbox gap={4}>
                    <Button
                      onClick={() => handleRemoveCloudFolder(folder.id)}
                      relevancy="danger"
                      bSize="small"
                    >
                      Remove
                    </Button>
                    <Button
                      onClick={async () => {
                        try {
                          await cloudDatabase.discoverCloudFolderTracks(folder.id);
                          // await cloudDatabase.syncCloudTracksMetadata(folder.provider_type as CloudProviderType);
                          toastsAPI.add('success', 'Folder fetch  completed successfully');
                        } catch (error) {
                          console.error('Failed to fetch folder:', error);
                          toastsAPI.add('danger', 'Failed to fetch folder. Please try again.');
                        }
                      }}
                      bSize="small"
                    >
                      Fetch
                    </Button>
                  </Flexbox>
                </li>
              ))}
            </ul>
          </div>
        )}
      </Setting.Section>

      {showCloudFolderSelect && (
        <div className={styles.modal}>
          <CloudFolderSelect
            providerType="dropbox"
            onSelect={handleCloudFolderSelect}
            onCancel={() => setShowCloudFolderSelect(false)}
          />
        </div>
      )}
    </div>
  );
}
