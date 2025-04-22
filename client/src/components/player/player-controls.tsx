import { motion } from "motion/react";
import {
  Pause,
  Play,
  Repeat,
  Shuffle,
  SkipBack,
  SkipForward,
} from "lucide-react";
import { Button } from "../ui/button";
import { cn } from "../../lib/utils";
import { AutoRepeatMode, IO, TrackInfo } from "../../lib/socket-io";

// Repeat mode icons for different states
const REPEAT_MODE_ICONS = {
  [AutoRepeatMode.None]: <Repeat className="size-4" />,
  [AutoRepeatMode.List]: (
    <Repeat className="size-4 animate-pulse text-primary" />
  ),
  [AutoRepeatMode.Track]: (
    <div className="flex-flex-col gap-0.5">
      <Repeat className="size-4 animate-pulse text-primary" />
      <p className="font-medium">1</p>
    </div>
  ),
};

// Order of repeat modes when cycling
const REPEAT_MODE_CYCLE = [
  AutoRepeatMode.None,
  AutoRepeatMode.List,
  AutoRepeatMode.Track,
];

type PlayerControlsProps = {
  track: TrackInfo;
  io: IO;
};
export function PlayerControls({ track, io }: PlayerControlsProps) {
  const PlayPauseButton = track.is_playing ? Pause : Play;

  function cycleRepeatMode() {
    const currentMode = track?.auto_repeat_mode;
    if (!currentMode) return;

    const currentIndex = REPEAT_MODE_CYCLE.indexOf(currentMode);
    const nextIndex = (currentIndex + 1) % REPEAT_MODE_CYCLE.length;
    const nextMode = REPEAT_MODE_CYCLE[nextIndex];

    io.setRepeatMode(nextMode);
  }

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0 }}
      className="flex gap-4 sm:gap-8 w-full items-center text-shadow-md justify-center"
    >
      {/* Shuffle button */}
      {track.shuffle != null && (
        <Button
          onClick={() => io.toggleShuffle()}
          variant="ghost"
          className={cn(
            "rounded-full size-8! text-muted-foreground",
            track.shuffle && "text-primary",
          )}
        >
          <Shuffle className="size-5" />
        </Button>
      )}

      {/* Previous track button */}
      <Button
        onClick={() => io.previousTrack()}
        variant="ghost"
        className="rounded-full size-10 sm:size-13!"
      >
        <SkipBack className="fill-white size-7 sm:size-10" />
      </Button>

      {/* Play/Pause button */}
      <Button
        onClick={() => io.togglePlayPause()}
        variant="ghost"
        className="rounded-full bg-white size-12 sm:size-13 hover:bg-white!"
      >
        <PlayPauseButton className="fill-black stroke-black stroke-1 size-5 sm:size-8" />
      </Button>

      {/* Next track button */}
      <Button
        onClick={() => io.nextTrack()}
        variant="ghost"
        className="rounded-full size-10 sm:size-13!"
      >
        <SkipForward className="fill-white size-7 sm:size-10" />
      </Button>

      {/* Repeat mode button */}
      {!!track.auto_repeat_mode && (
        <Button
          onClick={cycleRepeatMode}
          variant="ghost"
          className={cn(
            "rounded-full size-8! text-muted-foreground",
            track.auto_repeat_mode !== AutoRepeatMode.None && "text-primary",
          )}
        >
          {REPEAT_MODE_ICONS[track.auto_repeat_mode]}
        </Button>
      )}
    </motion.div>
  );
}
