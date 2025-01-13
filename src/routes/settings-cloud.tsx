import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import styles from './settings.module.css';

interface CloudProvider {
  id: string;
  provider_type: string;
  created_at: number;
}

interface CloudFolder {
  id: string;
  provider_id: string;
  cloud_folder_id: string;
  cloud_folder_name: string;
  local_folder_path: string;
}

interface CloudFile {
  id: string;
  name: string;
  parent_id: string | null;
  is_folder: boolean;
}

export default function SettingsCloud() {
  const [isDropboxAuthorized, setIsDropboxAuthorized] = useState(false);
  const [cloudFolders, setCloudFolders] = useState<CloudFolder[]>([]);
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null);
  const [selectedCloudFolder, setSelectedCloudFolder] = useState<CloudFile | null>(null);
  const [cloudFolderList, setCloudFolderList] = useState<CloudFile[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    checkDropboxAuth();
    loadCloudFolders();
  }, []);

  const checkDropboxAuth = async () => {
    const isAuthorized = await invoke('cloud_dropbox_is_authorized');
    setIsDropboxAuthorized(isAuthorized as boolean);
  };

  const loadCloudFolders = async () => {
    // TODO: Implement loading cloud folders from database
  };

  const handleConnectDropbox = async () => {
    try {
      setIsLoading(true);
      const authUrl = await invoke('cloud_dropbox_start_auth');
      // Open auth URL in browser
      await invoke('open_browser', { url: authUrl });
      // TODO: Handle auth callback
    } catch (error) {
      console.error('Failed to connect to Dropbox:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDisconnectDropbox = async () => {
    try {
      setIsLoading(true);
      await invoke('cloud_dropbox_unauthorize');
      setIsDropboxAuthorized(false);
      setSelectedProvider(null);
    } catch (error) {
      console.error('Failed to disconnect Dropbox:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSelectProvider = async (providerId: string) => {
    setSelectedProvider(providerId);
    try {
      setIsLoading(true);
      const files = await invoke('cloud_dropbox_list_files', { folderId: '' });
      setCloudFolderList(files as CloudFile[]);
    } catch (error) {
      console.error('Failed to list cloud folders:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSelectCloudFolder = (folder: CloudFile) => {
    setSelectedCloudFolder(folder);
  };

  const handleAddFolderMapping = async () => {
    if (!selectedProvider || !selectedCloudFolder) return;

    try {
      setIsLoading(true);
      // Open folder picker
      const localPath = await open({
        directory: true,
        multiple: false,
        title: 'Select Local Music Folder',
      });

      if (!localPath) return;

      // TODO: Save folder mapping to database
      await loadCloudFolders();
    } catch (error) {
      console.error('Failed to add folder mapping:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRemoveFolderMapping = async (folderId: string) => {
    try {
      setIsLoading(true);
      // TODO: Remove folder mapping from database
      await loadCloudFolders();
    } catch (error) {
      console.error('Failed to remove folder mapping:', error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className={styles.settingsSection}>
      <h2>Cloud Storage</h2>

      {/* Cloud Providers */}
      <div className={styles.settingsGroup}>
        <h3>Cloud Providers</h3>
        <div className={styles.providerList}>
          <div className={styles.provider}>
            <img src="/dropbox-logo.png" alt="Dropbox" />
            <div className={styles.providerInfo}>
              <h4>Dropbox</h4>
              <p>{isDropboxAuthorized ? 'Connected' : 'Not connected'}</p>
            </div>
            <button
              onClick={isDropboxAuthorized ? handleDisconnectDropbox : handleConnectDropbox}
              disabled={isLoading}
            >
              {isDropboxAuthorized ? 'Disconnect' : 'Connect'}
            </button>
          </div>
        </div>
      </div>

      {/* Folder Mappings */}
      {isDropboxAuthorized && (
        <div className={styles.settingsGroup}>
          <h3>Folder Mappings</h3>
          
          {/* Add New Mapping */}
          <div className={styles.addMapping}>
            <h4>Add New Mapping</h4>
            <div className={styles.folderSelector}>
              <div className={styles.cloudFolderList}>
                {cloudFolderList.map((folder) => (
                  <div
                    key={folder.id}
                    className={`${styles.cloudFolder} ${
                      selectedCloudFolder?.id === folder.id ? styles.selected : ''
                    }`}
                    onClick={() => handleSelectCloudFolder(folder)}
                  >
                    <span className={folder.is_folder ? styles.folderIcon : styles.fileIcon} />
                    {folder.name}
                  </div>
                ))}
              </div>
              <button
                onClick={handleAddFolderMapping}
                disabled={!selectedCloudFolder || isLoading}
              >
                Add Mapping
              </button>
            </div>
          </div>

          {/* Existing Mappings */}
          <div className={styles.mappingList}>
            {cloudFolders.map((folder) => (
              <div key={folder.id} className={styles.mapping}>
                <div className={styles.mappingInfo}>
                  <div className={styles.cloudPath}>
                    <span className={styles.folderIcon} />
                    {folder.cloud_folder_name}
                  </div>
                  <div className={styles.arrow}>→</div>
                  <div className={styles.localPath}>
                    <span className={styles.folderIcon} />
                    {folder.local_folder_path}
                  </div>
                </div>
                <button
                  onClick={() => handleRemoveFolderMapping(folder.id)}
                  className={styles.removeButton}
                  disabled={isLoading}
                >
                  Remove
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
