import { useCallback, useState, useEffect } from 'react';
import Button from '../elements/Button';
import Flexbox from '../elements/Flexbox';
import * as Setting from '../components/Setting';
import { cloud } from '../lib/cloud-provider';
import { useToastsAPI } from '../stores/useToastsStore';
import CloudFolderSelect from '../components/CloudFolderSelect';
import { open } from '@tauri-apps/plugin-shell';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import useCloudLibraryStore, { useCloudAPI } from '../stores/useCloudLibraryStore';
import type { CloudFile } from '../generated/typings';
import { cloudDatabase } from '../lib/cloud-database';
import { cloudMetadata } from '../lib/cloud-metadata';

import styles from './settings-cloud.module.css';

export default function SettingsCloud() {
  const [isAuthorized, setIsAuthorized] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const [showCloudFolderSelect, setShowCloudFolderSelect] = useState(false);
  const [authCode, setAuthCode] = useState('');
  const [showAuthInput, setShowAuthInput] = useState(false);
  
  const toastsAPI = useToastsAPI();
  const cloudAPI = useCloudAPI();
  const { folders, isSyncing } = useCloudLibraryStore();

  useEffect(() => {
    // Check initial authorization status
    cloud.dropboxIsAuthorized().then(setIsAuthorized);
    // Load existing cloud folders
    if (isAuthorized) {
      cloudAPI.loadFolders();
    }
  }, [isAuthorized, cloudAPI]);

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
      await cloudAPI.loadFolders();
    } catch (error) {
      console.error('Failed to connect to Dropbox:', error);
      toastsAPI.add('danger', 'Failed to connect to Dropbox. Please try again.');
    } finally {
      setIsConnecting(false);
    }
  }, [authCode, toastsAPI, cloudAPI]);

  const handleAuthCancel = useCallback(() => {
    setShowAuthInput(false);
    setAuthCode('');
    setIsConnecting(false);
  }, []);

  const handleDisconnect = useCallback(async () => {
    try {
      await cloud.dropboxUnauthorize();
      setIsAuthorized(false);
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

      const folder = {
        id: crypto.randomUUID(),
        provider_type: 'dropbox' as const,
        cloud_folder_id: cloudFile.id,
        cloud_folder_path: fullPath,
        local_folder_path: localPath,
      };

      await cloudAPI.saveFolder(folder);
      setShowCloudFolderSelect(false);
    } catch (error) {
      console.error('Failed to add cloud folder:', error);
      toastsAPI.add('danger', 'Failed to add cloud folder. Please try again.');
    }
  }, [toastsAPI, cloudAPI]);

  const handleMetadataSync = async () => {
    try {
      const syncResult = await cloudMetadata.syncCloudMetadata();
      const updateResult = await cloudMetadata.updateCloudMetadata();
      
      let message = '';
      if (syncResult.is_fresh_start) {
        message = `Initial metadata sync complete. Created ${syncResult.tracks_created} tracks.`;
      } else {
        message = `Metadata sync complete. Updated ${syncResult.tracks_updated} tracks, created ${syncResult.tracks_created} tracks.`;
      }
      message += ` ${updateResult.tracks_included} tracks included in cloud metadata, ${updateResult.tracks_skipped} skipped.`;
      
      toastsAPI.add('success', message);
    } catch (error) {
      console.error('Failed to sync metadata:', error);
      toastsAPI.add('danger', 'Failed to sync cloud metadata. Please try again.');
    }
  };

  return (
    <div className={styles.container}>
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
                <Button onClick={cloudAPI.syncMetadata} disabled={isSyncing}>
                  {isSyncing ? 'Syncing...' : 'Sync Metadata'}
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

          {(
            <div className={styles.cloudFolders}>
              <h3>Sync Folders</h3>
              <Flexbox gap={4} className={styles.addFolderSection}>
                <Button onClick={handleAddCloudFolder}>
                  Add Cloud Folder
                </Button>
              </Flexbox>
              <ul>
                {folders.map(folder => (
                  <li key={folder.id}>
                    <span>{folder.cloud_folder_path} → {folder.local_folder_path}</span>
                    <Flexbox gap={4}>
                      <Button
                        onClick={() => cloudAPI.removeFolder(folder.id)}
                        relevancy="danger"
                        bSize="small"
                      >
                        Remove
                      </Button>
                      <Button
                        onClick={() => cloudAPI.discoverFolderTracks(folder.id)}
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
    </div>
  );
}
