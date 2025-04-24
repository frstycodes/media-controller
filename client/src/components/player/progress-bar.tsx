import { useEffect, useState } from "react";
import { motion } from "motion/react";
import { Slider } from "../ui/slider";
import { cn } from "@/lib/utils";
import { IO, TrackTimeline, events } from "@/lib/socket-io";

type ProgressBarProps = {
  io: IO;
  active: boolean;
  duration: number;
};

export function ProgressBar({ io, active, duration }: ProgressBarProps) {
  const [progress, setProgress] = useState(0);
  const [seeking, setSeeking] = useState(false);

  useEffect(() => {
    const socket = io.getSocket();
    const onTrackTimeline = (timeline: TrackTimeline) => {
      if (seeking) return false;
      console.log("hello", timeline.progress);
      setProgress(timeline.progress);
    };

    socket.on(events.TRACK_TIMELINE, onTrackTimeline);
    return () => {
      socket.off(events.TRACK_TIMELINE, onTrackTimeline);
    };
  }, [io, seeking]);

  function handleSeek([newVal]: [number]) {
    setSeeking(false);
    io.seek(newVal);
  }

  const sliderProps = {
    thumbCn: !active ? "hidden" : "",
    trackCn: !active ? "h-0.5!" : "",
  };

  return (
    <motion.div layout="position">
      <div>
        <Slider
          min={0}
          max={duration}
          value={[progress]}
          onValueChange={([newVal]) => {
            setSeeking(true);
            setProgress(newVal);
          }}
          onValueCommit={handleSeek}
          {...sliderProps}
        />
        <div
          className={cn(
            "flex text-sm text-muted-foreground text-shadow-md justify-between py-2",
            !active && "text-[0px]",
          )}
        >
          <span>{formatMilliseconds(progress)}</span>
          <span>{formatMilliseconds(duration)}</span>
        </div>
      </div>
    </motion.div>
  );
}

function formatMilliseconds(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainingSeconds = seconds % 60;

  const formattedHours = hours > 0 ? `${hours}:` : "";
  const formattedMinutes =
    minutes < 10 && hours > 0 ? `0${minutes}:` : `${minutes}:`;
  const formattedSeconds =
    remainingSeconds < 10 ? `0${remainingSeconds}` : `${remainingSeconds}`;

  return `${formattedHours}${formattedMinutes}${formattedSeconds}`;
}
