import * as Slider from '@radix-ui/react-slider';
import { useCallback, useState } from 'react';

import type { Track } from '../generated/typings';
import usePlayingTrackCurrentTime from '../hooks/usePlayingTrackCurrentTime';
import { usePlayerAPI } from '../stores/usePlayerStore';

import useFormattedDuration from '../hooks/useFormattedDuration';
import styles from './TrackProgress.module.css';

type Props = {
  trackPlaying: Track;
};

export default function TrackProgress(props: Props) {
  const { trackPlaying } = props;

  const elapsed = usePlayingTrackCurrentTime();
  const playerAPI = usePlayerAPI();
  const [tempPosition, setTempPosition] = useState<number | null>(null);
  const [tooltipTargetTime, setTooltipTargetTime] = useState<null | number>(null);
  const [tooltipX, setTooltipX] = useState<null | number>(null);

  const jumpAudioTo = useCallback(
    (values: number[]) => {
      const [to] = values;
      playerAPI.jumpTo(to);
      setTempPosition(null);
    },
    [playerAPI],
  );

  const handleSliderChange = useCallback((values: number[]) => {
    const [to] = values;
    setTempPosition(to);
    setTooltipTargetTime(to);
    const percent = (to / trackPlaying.duration) * 100;
    setTooltipX(percent);
  }, [trackPlaying.duration]);

  const showTooltip = useCallback(
    (e: React.MouseEvent<HTMLElement>) => {
      if (tempPosition !== null) return; // Don't update tooltip if dragging
      
      const { offsetX } = e.nativeEvent;
      const barWidth = e.currentTarget.offsetWidth;
      const percent = (offsetX / barWidth) * 100;
      const time = (percent * trackPlaying.duration) / 100;

      setTooltipTargetTime(time);
      setTooltipX(percent);
    },
    [trackPlaying.duration, tempPosition],
  );

  const hideTooltip = useCallback(() => {
    if (tempPosition !== null) return; // Don't hide tooltip if dragging
    setTooltipTargetTime(null);
    setTooltipX(null);
  }, [tempPosition]);

  const tooltipContent = useFormattedDuration(tooltipTargetTime);
  const currentPosition = tempPosition ?? elapsed;

  return (
    <Slider.Root
      min={0}
      max={trackPlaying.duration}
      step={1}
      value={[currentPosition]}
      onValueChange={handleSliderChange}
      onValueCommit={jumpAudioTo}
      className={styles.trackRoot}
      onMouseMoveCapture={showTooltip}
      onMouseLeave={hideTooltip}
    >
      <Slider.Track className={styles.trackProgress}>
        <Slider.Range className={styles.trackRange} />
        <div
          className={styles.progressTooltip}
          style={{
            left: `${tooltipX}%`,
            display: tooltipX == null ? 'none' : 'block',
          }}
        >
          {tooltipContent}
        </div>
      </Slider.Track>
      <Slider.Thumb />
    </Slider.Root>
  );
}
