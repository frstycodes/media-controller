import { useEffect, useState } from "react";
import { motion } from "motion/react";
import { Slider } from "../ui/slider";
import { cn, formatMilliseconds } from "@/lib/utils";
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
    const onTrackTimeline = (timeline: TrackTimeline) => {
      if (seeking) return false;
      setProgress(timeline.progress);
    };

    io.socket.on(events.TRACK_TIMELINE, onTrackTimeline);
    return () => {
      io.socket.off(events.TRACK_TIMELINE, onTrackTimeline);
    };
  }, [io, seeking]);

  function handleSeek([newVal]: [number]) {
    setSeeking(false);
    io.seek(newVal);
  }

  return (
    <motion.div layout="position">
      <Slider
        min={0}
        max={duration}
        value={[progress]}
        onValueChange={([newVal]) => {
          setSeeking(true);
          setProgress(newVal);
        }}
        onValueCommit={handleSeek}
        trackCn={cn("bg-white/15", !active && "h-0.5!")}
        thumbCn={cn(!active && "hidden")}
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
    </motion.div>
  );
}
