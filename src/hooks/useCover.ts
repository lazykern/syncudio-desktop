import { useEffect, useState } from 'react';

import type { Track } from '../generated/typings';
import { getCover } from '../lib/cover';
import { getTrackPath } from '../lib/track';

/**
 * Given a track, get its associated cover as an Image src
 */
export default function useCover(track: Track): string | null {
  const [coverPath, setCoverPath] = useState<string | null>(null);

  useEffect(() => {
    const refreshCover = async () => {
      const path = getTrackPath(track);
      const cover = await getCover(path);
      setCoverPath(cover);
    };

    refreshCover();
  }, [track]);

  return coverPath;
}
