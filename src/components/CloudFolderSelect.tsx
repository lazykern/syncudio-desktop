import { useCallback, useEffect, useState } from 'react';
import { cloud } from '../lib/cloud-provider';
import type { CloudFile, CloudProviderType } from '../generated/typings';
import Flexbox from '../elements/Flexbox';
import Button from '../elements/Button';
import styles from './CloudFolderSelect.module.css';

interface CloudFolderSelectProps {
  providerType: CloudProviderType;
  onSelect: (folder: CloudFile, fullPath: string) => void;
  onCancel: () => void;
}

interface BreadcrumbItem {
  file: CloudFile | null;
  name: string;
}

export default function CloudFolderSelect({ providerType, onSelect, onCancel }: CloudFolderSelectProps) {
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentFolder, setCurrentFolder] = useState<CloudFile | null>(null);
  const [selectedFolder, setSelectedFolder] = useState<CloudFile | null>(null);
  const [folders, setFolders] = useState<CloudFile[]>([]);
  const [breadcrumbs, setBreadcrumbs] = useState<BreadcrumbItem[]>([
    { file: null, name: 'Root' }
  ]);

  const loadFolders = useCallback(async (folder: CloudFile | null) => {
    try {
      setIsLoading(true);
      setError(null);
      setSelectedFolder(null);

      const files = folder 
        ? await cloud.listFiles(providerType, folder.id, false)
        : await cloud.listRootFiles(providerType, false);

      const folderFiles = files.filter(file => file.is_folder);
      setFolders(folderFiles);
    } catch (err) {
      setError('Failed to load folders. Please try again.');
      console.error('Error loading folders:', err);
    } finally {
      setIsLoading(false);
    }
  }, [providerType]);

  useEffect(() => {
    loadFolders(null);
  }, [loadFolders]);

  const handleFolderOpen = useCallback(async (folder: CloudFile) => {
    setCurrentFolder(folder);
    setBreadcrumbs(prev => [...prev, { file: folder, name: folder.name }]);
    await loadFolders(folder);
  }, [loadFolders]);

  const handleBreadcrumbClick = useCallback(async (item: BreadcrumbItem, index: number) => {
    setCurrentFolder(item.file);
    setBreadcrumbs(prev => prev.slice(0, index + 1));
    await loadFolders(item.file);
  }, [loadFolders]);

  const getFullPath = useCallback((folder: CloudFile) => {
    const path = breadcrumbs
      .slice(1) // Skip 'Root'
      .map(item => item.name)
      .concat(folder.name)
      .join('/');
    return `/${path}`;
  }, [breadcrumbs]);

  const handleFolderSelect = useCallback((folder: CloudFile) => {
    setSelectedFolder(folder);
  }, []);

  const handleConfirmSelection = useCallback(() => {
    if (selectedFolder) {
      onSelect(selectedFolder, getFullPath(selectedFolder));
    }
  }, [selectedFolder, onSelect, getFullPath]);

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2>Select Cloud Folder</h2>
        <div className={styles.breadcrumbs}>
          {breadcrumbs.map((item, index) => (
            <span key={item.file?.id ?? 'root'}>
              {index > 0 && <span className={styles.separator}>/</span>}
              <button
                className={styles.breadcrumbButton}
                onClick={() => handleBreadcrumbClick(item, index)}
              >
                {item.name}
              </button>
            </span>
          ))}
        </div>
      </div>

      <div className={styles.folderList}>
        {isLoading ? (
          <div className={styles.message}>Loading folders...</div>
        ) : error ? (
          <div className={styles.error}>{error}</div>
        ) : folders.length === 0 ? (
          <div className={styles.message}>No folders found</div>
        ) : (
          <div className={styles.grid}>
            {folders.map(folder => (
              <button
                key={folder.id}
                className={`${styles.folderItem} ${folder.id === selectedFolder?.id ? styles.selected : ''}`}
                onClick={() => handleFolderSelect(folder)}
                onDoubleClick={() => handleFolderOpen(folder)}
              >
                <div className={styles.folderIcon}>📁</div>
                <div className={styles.folderName}>{folder.name}</div>
              </button>
            ))}
          </div>
        )}
      </div>

      <Flexbox className={styles.actions}>
        <Button onClick={onCancel}>Cancel</Button>
        <Button 
          onClick={handleConfirmSelection}
          disabled={!selectedFolder}
        >
          Select Folder
        </Button>
      </Flexbox>
    </div>
  );
} 