import { useLoaderData } from 'react-router';

import * as Setting from '../components/Setting';
import CheckboxSetting from '../components/SettingCheckbox';
import Button from '../elements/Button';
import ExternalButton from '../elements/ExternalButton';
import ExternalLink from '../elements/ExternalLink';
import Flexbox from '../elements/Flexbox';
import Heart from '../elements/Heart';
import useInvalidate, { useInvalidateCallback } from '../hooks/useInvalidate';
import SettingsAPI from '../stores/SettingsAPI';
import type { SettingsLoaderData } from './settings';

export default function ViewSettingsAbout() {
  const { config, version, tauriVersion, appStorageDir } =
    useLoaderData() as SettingsLoaderData;

  const invalidate = useInvalidate();

  return (
    <div className="setting setting-about">
      <Setting.Section>
        <Setting.Title>About Syncudio</Setting.Title>
        <Setting.Description>
          Syncudio {version} - A fork of{' '}
          <ExternalLink href="https://github.com/martpie/museeks" type="url">
            Museeks
          </ExternalLink>
          {' with cloud sync capabilities. - '}
          <ExternalLink
            href="https://github.com/lazykern/syncudio-desktop"
            type="url"
          >
            GitHub
          </ExternalLink>
        </Setting.Description>
        <CheckboxSetting
          slug="update"
          title="Automatically check for updates"
          value={config.auto_update_checker}
          onChange={useInvalidateCallback(SettingsAPI.toggleAutoUpdateChecker)}
        />
        <div>
          <Button
            onClick={() => {
              SettingsAPI.checkForUpdate().then(invalidate);
            }}
          >
            Check for update
          </Button>
        </div>
      </Setting.Section>
      <Setting.Section>
        <Setting.Title>Credits</Setting.Title>
        <Setting.Description>
          Based on <ExternalLink href="https://github.com/martpie/museeks" type="url">Museeks</ExternalLink>{' '}
          by Pierre de la Martinière (
          <ExternalLink href="https://martpie.io" type="url">
            @martpie
          </ExternalLink>
          ) and its{' '}
          <ExternalLink
            href="https://github.com/martpie/museeks/graphs/contributors"
            type="url"
          >
            contributors
          </ExternalLink>
          .
        </Setting.Description>
      </Setting.Section>
      <Setting.Section>
        <Setting.Title>Report issue / Ask for a feature</Setting.Title>
        <Setting.Description>
          Bugs happen. Please, do not hesitate to report them or to ask for
          features you would like to see, using the{' '}
          <ExternalLink
            href="http://github.com/martpie/Museeks/issues"
            type="url"
          >
            issue tracker
          </ExternalLink>
          .
        </Setting.Description>
      </Setting.Section>
      <Setting.Section>
        <Setting.Title>Internals</Setting.Title>
        <Setting.Description>Tauri {tauriVersion}</Setting.Description>
        <Flexbox gap={4}>
          <ExternalButton href={appStorageDir} type="filedir">
            Open storage directory
          </ExternalButton>
        </Flexbox>
      </Setting.Section>
    </div>
  );
}
