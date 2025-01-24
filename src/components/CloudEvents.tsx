import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import useToastsStore from '../stores/useToastsStore';
import type { TrackDownloadedPayload } from '../generated/typings';

/**
 * Handle cloud-related events like downloads, uploads, and sync operations
 */
function CloudEvents() {
  const queryClient = useQueryClient();
  const { api: toastApi } = useToastsStore();

  useEffect(() => {
    const unlisteners = [
      // Track download events
      listen<TrackDownloadedPayload>('track-downloaded', ({ payload }) => {
        // Show success notification
        toastApi.add(
          'success',
          `Downloaded "${payload.relative_path.split('/').pop() || ''}"`,
          5000,
        );

        // Invalidate relevant queries
        queryClient.invalidateQueries({ 
          queryKey: ['cloud', 'track', 'sync']
        });
        queryClient.invalidateQueries({ 
          queryKey: ['unified-tracks']
        });
      }),
    ];

    return () => {
      Promise.all(unlisteners).then(u => u.forEach(fn => fn()));
    };
  }, [queryClient, toastApi]);

  return null;
}

export default CloudEvents;
