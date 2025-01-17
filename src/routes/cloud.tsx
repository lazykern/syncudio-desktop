import { useEffect, useState } from 'react';
import styles from './cloud.module.css';
import Flexbox from '../elements/Flexbox';
import Button from '../elements/Button';
import { cloudDatabase } from '../lib/cloud-database';
import type { CloudFolder } from '../generated/typings';

function CloudHeader() {
  return (
    <div className={styles.header}>
      <Flexbox gap={4} align="center">
        <div className={styles.syncStatus}>
          <h2>Cloud Sync</h2>
          <span className={styles.statusIndicator}>All synced</span>
        </div>
        <div className={styles.globalActions}>
          <Button>Force Sync All</Button>
          <Button>Pause Sync</Button>
        </div>
      </Flexbox>
    </div>
  );
}

function CloudSidebar({ folders }: { folders: CloudFolder[] }) {
  return (
    <div className={styles.sidebar}>
      <h3>Sync Folders</h3>
      <ul className={styles.folderList}>
        {folders.map(folder => (
          <li key={folder.id} className={styles.folderItem}>
            <div className={styles.folderInfo}>
              <span className={styles.folderName}>{folder.cloud_folder_path}</span>
              <span className={styles.folderStatus}>Synced</span>
            </div>
            <div className={styles.folderActions}>
              <Button bSize="small">Sync</Button>
            </div>
          </li>
        ))}
      </ul>
    </div>
  );
}

function CloudContent() {
  const [trackPaths, setTrackPaths] = useState<{ id: string; relative_path: string }[]>([]);

  return (
    <div className={styles.content}>
      <div className={styles.trackGrid}>
        <div className={styles.trackGridHeader}>
          <div>Name</div>
          <div>Status</div>
          <div>Last Synced</div>
          <div>Actions</div>
        </div>
        <div className={styles.trackList}>
          {trackPaths.map(path => (
            <div key={path.id} className={styles.trackItem}>
              <div className={styles.trackName}>{path.relative_path}</div>
              <div className={styles.trackStatus}>Synced</div>
              <div className={styles.trackLastSync}>2 mins ago</div>
              <div className={styles.trackActions}>
                <Button bSize="small">Force Sync</Button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function QueueStatusBar() {
  return (
    <div className={styles.queueStatus}>
      <div className={styles.queueInfo}>
        <span>Queue: 0 uploads, 0 downloads</span>
      </div>
      <div className={styles.queueProgress}>
        <div className={styles.progressBar} style={{ width: '0%' }} />
      </div>
    </div>
  );
}

export default function ViewCloud() {
  const [folders, setFolders] = useState<CloudFolder[]>([]);

  useEffect(() => {
    // Load cloud folders
    const loadFolders = async () => {
      try {
        const allFolders = await cloudDatabase.getCloudFoldersByProvider('dropbox');
        setFolders(allFolders);
      } catch (error) {
        console.error('Failed to load cloud folders:', error);
      }
    };

    loadFolders();
  }, []);

  return (
    <div className={styles.container}>
      <CloudHeader />
      <div className={styles.main}>
        <CloudSidebar folders={folders} />
        <CloudContent />
      </div>
      <QueueStatusBar />
    </div>
  );
}

