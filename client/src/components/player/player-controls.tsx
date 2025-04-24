import { motion } from "motion/react";
import {
  Pause,
  Play,
  Repeat,
  Repeat1,
  Shuffle,
  SkipBack,
  SkipForward,
} from "lucide-react";
import { Button } from "../ui/button";
import { cn } from "../../lib/utils";
import { AutoRepeatMode, events, IO, TrackControls } from "@/lib/socket-io";
import { useEffect, useState } from "react";

// Repeat mode icons for different states
const REPEAT_MODE_ICONS = {
  [AutoRepeatMode.None]: <Repeat className="size-5" />,
  [AutoRepeatMode.List]: <Repeat className="size-5 text-primary" />,
  [AutoRepeatMode.Track]: <Repeat1 className="size-5  text-primary" />,
};

// Order of repeat modes when cycling
const REPEAT_MODE_CYCLE = [
  AutoRepeatMode.None,
  AutoRepeatMode.List,
  AutoRepeatMode.Track,
];

type PlayerControlsProps = {
  io: IO;
  active: boolean;
};
export function PlayerControls({ io, active }: PlayerControlsProps) {
  const [controls, setControls] = useState<TrackControls | null>(null);

  function cycleRepeatMode() {
    if (!controls) return;
    const idx = REPEAT_MODE_CYCLE.indexOf(controls.auto_repeat_mode);
    const nextIdx = (idx + 1) % REPEAT_MODE_CYCLE.length;
    const nextMode = REPEAT_MODE_CYCLE[nextIdx];

    io.setRepeatMode(nextMode);
  }

  useEffect(() => {
    const socket = io.getSocket();
    socket.on(events.TRACK_CONTROLS, setControls);
    return () => {
      socket.off(events.TRACK_CONTROLS, setControls);
    };
  }, [io]);

  if (!controls) return null;

  const PlayPauseButton = controls?.playing ? Pause : Play;

  if (!active) return null;
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0 }}
      className="flex gap-4 sm:gap-8 w-full items-center text-shadow-md justify-center"
    >
      {/* Shuffle button */}
      {controls.shuffle_enabled && (
        <Button
          onClick={() => io.toggleShuffle()}
          variant="ghost"
          className={cn(
            "rounded-full size-8! text-muted-foreground hover:bg-transparent",
            controls.shuffle && "text-primary",
          )}
        >
          <Shuffle className="size-5" />
        </Button>
      )}

      {/* Previous track button */}
      {controls.prev_enabled && (
        <Button
          onClick={() => io.previousTrack()}
          variant="ghost"
          className="rounded-full size-10 opacity-50 hover:opacity-100 sm:size-13! hover:bg-transparent"
        >
          <SkipBack className="fill-white size-7 drop-shadow-md sm:size-10" />
        </Button>
      )}

      {/* Play/Pause button */}
      {controls.play_pause_enabled && (
        <Button
          onClick={() => io.togglePlayPause()}
          variant="ghost"
          className="rounded-full bg-white size-12 sm:size-13 hover:bg-white!"
        >
          <PlayPauseButton className="fill-black stroke-black stroke-1 size-5 sm:size-8" />
        </Button>
      )}

      {/* Next track button */}
      {controls.next_enabled && (
        <Button
          onClick={() => io.nextTrack()}
          variant="ghost"
          className="rounded-full hover:bg-transparent size-10 sm:size-13! opacity-50 hover:opacity-100"
        >
          <SkipForward className="fill-white size-7 sm:size-10 drop-shadow-md" />
        </Button>
      )}
      {/* Repeat mode button */}
      {controls.auto_repeat_mode_enabled && (
        <Button
          onClick={cycleRepeatMode}
          variant="ghost"
          className={cn(
            "rounded-full size-8! hover:bg-transparent text-muted-foreground",
            controls.auto_repeat_mode !== AutoRepeatMode.None && "text-primary",
          )}
        >
          {REPEAT_MODE_ICONS[controls.auto_repeat_mode]}
        </Button>
      )}
    </motion.div>
  );
}
