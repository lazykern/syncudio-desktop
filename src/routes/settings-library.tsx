import { useLoaderData } from 'react-router';

import * as Setting from '../components/Setting';
import CheckboxSetting from '../components/SettingCheckbox';
import Button from '../elements/Button';
import Flexbox from '../elements/Flexbox';
import useInvalidate, { useInvalidateCallback } from '../hooks/useInvalidate';
import SettingsAPI from '../stores/SettingsAPI';
import useLibraryStore, { useLibraryAPI } from '../stores/useLibraryStore';
import type { SettingsLoaderData } from './settings';

import { open } from '@tauri-apps/plugin-dialog';
import { useCallback, useEffect, useState } from 'react';
import styles from './settings-library.module.css';
import database from '../lib/database';
import type { LocalFolder } from '../generated/typings';

export default function ViewSettingsLibrary() {
  const libraryAPI = useLibraryAPI();
  const isLibraryRefreshing = useLibraryStore((state) => state.refreshing);
  const isReindexing = useLibraryStore((state) => state.reindexing);
  const { config } = useLoaderData() as SettingsLoaderData;
  const invalidate = useInvalidate();
  const [localFolders, setLocalFolders] = useState<LocalFolder[]>([]);

  useEffect(() => {
    const loadFolders = async () => {
      const folders = await database.getLocalFolders();
      setLocalFolders(folders);
    };
    loadFolders();
  }, []);

  const addLibraryFolders = useCallback(async () => {
    const paths = await open({
      directory: true,
      multiple: true,
    });

    if (paths == null) {
      return;
    }

    await libraryAPI.addLibraryFolders(paths);
    const folders = await database.getLocalFolders();
    setLocalFolders(folders);
    invalidate();
  }, [libraryAPI.addLibraryFolders, invalidate]);

  const removeLibraryFolder = useCallback(async (path: string) => {
    await libraryAPI.removeLibraryFolder(path);
    const folders = await database.getLocalFolders();
    setLocalFolders(folders);
    invalidate();
  }, [libraryAPI.removeLibraryFolder, invalidate]);

  return (
    <div className="setting settings-musicfolder">
      <Setting.Section>
        <Setting.Title>Files</Setting.Title>
        {localFolders.length === 0 && (
          <Setting.Description>
            There are no folders in your library.
          </Setting.Description>
        )}
        {localFolders.length > 0 && (
          <ul className={styles.libraryFolders}>
            {localFolders.map((folder) => {
              return (
                <li key={folder.path}>
                  <Flexbox align="center">
                    <button
                      type="button"
                      className={styles.libraryFoldersRemove}
                      data-syncudio-action
                      onClick={() => removeLibraryFolder(folder.path)}
                    >
                      &times;
                    </button>
                    <span>{folder.path}</span>
                  </Flexbox>
                </li>
              );
            })}
          </ul>
        )}
        <Flexbox gap={4}>
          <Button
            disabled={isLibraryRefreshing}
            onClick={useInvalidateCallback(addLibraryFolders)}
          >
            Add folder
          </Button>
          <Button
            disabled={isLibraryRefreshing}
            onClick={useInvalidateCallback(libraryAPI.refresh)}
          >
            Refresh library
          </Button>
          <Button
            disabled={isReindexing}
            onClick={useInvalidateCallback(libraryAPI.reindex)}
          >
            Reindex tracks
          </Button>
        </Flexbox>
        <Setting.Description>
          <code>.m3u</code> files will also be imported as playlists.
        </Setting.Description>
      </Setting.Section>
      <Setting.Section>
        <CheckboxSetting
          slug="library-autorefresh"
          title="Automatically refresh library on startup"
          value={config.library_autorefresh}
          onChange={useInvalidateCallback(SettingsAPI.toggleLibraryAutorefresh)}
        />
      </Setting.Section>
      <Setting.Section>
        <Setting.Title>Danger zone</Setting.Title>
        <Setting.Description>
          Delete all tracks and playlists from Syncudio.
        </Setting.Description>
        <Flexbox>
          <Button
            relevancy="danger"
            title="Fully reset the library"
            disabled={isLibraryRefreshing}
            onClick={useInvalidateCallback(libraryAPI.reset)}
          >
            Reset library
          </Button>
        </Flexbox>
      </Setting.Section>
    </div>
  );
}
