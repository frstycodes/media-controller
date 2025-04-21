import "./App.css";
import { Socket, io as SocketIO } from "socket.io-client";
import { useEffect, useRef, useState } from "react";
import { events } from "./lib/socket-io";
import { Button } from "./components/ui/button";
import {
  Fullscreen,
  Pause,
  Play,
  Repeat,
  Shuffle,
  SkipBack,
  SkipForward,
} from "lucide-react";
import { Slider } from "./components/ui/slider";
import { AnimatePresence, motion } from "motion/react";
import { cn } from "./lib/utils";

const SOCKET_URL = "http://192.168.0.105:3001/ws";

type TrackInfo = {
  title: string;
  artist: string;
  album: string | null;
  duration: number;
  thumbnail: string;
  is_playing: boolean;
  shuffle: boolean;
  auto_repeat_mode: AutoRepeatMode;
};

type TrackProgress = {
  position: number;
  duration: number;
};

enum AutoRepeatMode {
  None = "none",
  Track = "track",
  List = "list",
}

const INACTIVITY_TIMEOUT = 10 * 1000;
const INTERACTION_EVENTS = [
  "mousedown",
  "mousemove",
  "keydown",
  "touchstart",
  "scroll",
] as const;

const repeat_mode_icons = {
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
const repeat_mode_cycle = [
  AutoRepeatMode.None,
  AutoRepeatMode.List,
  AutoRepeatMode.Track,
];

function App() {
  const [track, setTrack] = useState<TrackInfo | null>(null);
  const [active, setActive] = useState(true);
  const socketIO = useRef<Socket>(null);

  useEffect(() => {
    const controller = new AbortController();
    let inactivityTimer: number | undefined;

    const resetTimer = () => {
      clearTimeout(inactivityTimer);
      setActive(true);
      inactivityTimer = setTimeout(() => {
        setActive(false);
      }, INACTIVITY_TIMEOUT);
    };

    resetTimer();

    for (const ev of INTERACTION_EVENTS) {
      document.addEventListener(ev, resetTimer, { signal: controller.signal });
    }

    return () => {
      clearTimeout(inactivityTimer);
      controller.abort();
    };
  }, []);

  useEffect(() => {
    const io = SocketIO(SOCKET_URL);
    socketIO.current = io;
    io.emit(events.GET_MEDIA_DETAILS);
    io.on(events.MEDIA_DETAILS, setTrack);

    return () => {
      io.off(events.MEDIA_DETAILS);
      io.disconnect();
    };
  }, []);

  function togglePlayPause() {
    socketIO.current?.emit(events.TOGGLE_PLAY_PAUSE);
  }

  function nextTrack() {
    socketIO.current?.emit(events.NEXT_TRACK);
  }

  function previousTrack() {
    socketIO.current?.emit(events.PREVIOUS_TRACK);
  }

  function toggleShuffle() {
    socketIO.current?.emit(events.TOGGLE_SHUFFLE);
  }

  function setRepeatMode(mode: AutoRepeatMode) {
    socketIO.current?.emit(events.SET_REPEAT_MODE, mode);
  }

  function cycleRepeatMode() {
    const currentMode = track?.auto_repeat_mode;
    if (!currentMode) return;
    const nextMode =
      repeat_mode_cycle[
        (repeat_mode_cycle.indexOf(currentMode) + 1) % repeat_mode_cycle.length
      ];
    setRepeatMode(nextMode);
  }
  if (!track) return null;

  const PlayPauseButton = track.is_playing ? Pause : Play;
  return (
    <div className="flex-1 flex-col sm:flex-row flex justify-center items-center ">
      {active && (
        <Button
          onClick={() => {
            if (document.fullscreenElement) {
              document.exitFullscreen();
            } else {
              document.documentElement.requestFullscreen();
            }
          }}
          className="fixed z-10 top-0 left-0 text-muted-foreground"
          variant="ghost"
        >
          <Fullscreen />
        </Button>
      )}
      <img
        src={track.thumbnail}
        className="fixed inset-0 size-full opacity-20 blur-[100px]"
      />
      <div className="flex flex-col sm:flex-row items-center relative gap-8 w-full sm:max-w-[800px]">
        {/* THUMBNAIL */}
        <div className="relative grow sm:grow-0 w-full sm:w-[unset]">
          <img
            src={track.thumbnail}
            alt="thumbnail"
            className="absolute animate-pulse inset-0 blur-2xl w-full aspect-square!"
          />
          <img
            src={track.thumbnail}
            alt="thumbnail"
            className="w-full aspect-square sm:w-80 relative rounded-lg z-10 border-3 border-white/5"
          />
        </div>
        {/* CONTROLS  */}
        <div className="flex-1 flex w-full flex-col justify-center space-y-5">
          <motion.div layout="position">
            <p className="text-medium text-shadow-md text-left text-muted-foreground">
              {track.artist}
            </p>
            <h1
              className={cn(
                "font-extrabold transition-all text-left text-4xl text-shadow-lg",
                !active && "text-5xl",
              )}
            >
              {track.title}
            </h1>
          </motion.div>
          <div>
            <AnimatePresence mode="popLayout">
              {socketIO.current && (
                <motion.div layout="position">
                  <ProgressBar
                    io={socketIO.current}
                    hideNumbers={!active}
                    sliderProps={{
                      thumbCn: !active ? "hidden" : "",
                      trackCn: !active ? "h-0.5!" : "",
                    }}
                  />
                </motion.div>
              )}
              {active && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0 }}
                  className="flex gap-4 sm:gap-8 w-full items-center text-shadow-md justify-center"
                >
                  {track.shuffle != null && (
                    <Button
                      onClick={toggleShuffle}
                      variant="ghost"
                      className={cn(
                        "rounded-full size-8! text-muted-foreground",
                        track.shuffle && "text-primary",
                      )}
                    >
                      <Shuffle className="size-5" />
                    </Button>
                  )}
                  <Button
                    onClick={previousTrack}
                    variant="ghost"
                    className="rounded-full size-10 sm:size-13!"
                  >
                    <SkipBack className="fill-white size-7 sm:size-10" />
                  </Button>
                  <Button
                    onClick={togglePlayPause}
                    variant="ghost"
                    className="rounded-full bg-white size-12 sm:size-13 hover:bg-white!"
                  >
                    <PlayPauseButton className="fill-black stroke-black stroke-1 size-5 sm:size-8" />
                  </Button>
                  <Button
                    onClick={nextTrack}
                    variant="ghost"
                    className="rounded-full size-10 sm:size-13!"
                  >
                    <SkipForward className="fill-white size-7 sm:size-10" />
                  </Button>
                  {!!track.auto_repeat_mode && (
                    <Button
                      onClick={cycleRepeatMode}
                      variant="ghost"
                      className={cn(
                        "rounded-full size-8! text-muted-foreground",
                        track.auto_repeat_mode != AutoRepeatMode.None &&
                          "text-primary",
                      )}
                    >
                      {repeat_mode_icons[track.auto_repeat_mode]}
                    </Button>
                  )}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;

type ProgressBarProps = {
  io: Socket;
  hideNumbers?: boolean;
  sliderProps?: React.ComponentProps<typeof Slider>;
};
function ProgressBar(props: ProgressBarProps) {
  const [progress, setProgress] = useState(0);
  const [duration, setDuration] = useState(0);

  const [seeking, setSeeking] = useState(false);
  useEffect(() => {
    const onTrackProgress = (progress: TrackProgress) => {
      if (seeking) return false;
      setProgress(progress.position);
      setDuration(progress.duration);
    };
    props.io.on(events.TRACK_PROGRESS, onTrackProgress);

    return () => {
      props.io.off(events.TRACK_PROGRESS, onTrackProgress);
    };
  }, [props.io, seeking]);

  function handleSeek([newVal]: [number]) {
    setSeeking(false);
    props.io.emit(events.SEEK, { position: newVal });
  }
  return (
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
        {...(props.sliderProps ?? {})}
      />
      <div
        className={cn(
          "flex text-sm text-muted-foreground text-shadow-md justify-between py-2",
          props.hideNumbers && "text-[0px]",
        )}
      >
        <span>{formatMilliseconds(progress)}</span>
        <span>{formatMilliseconds(duration)}</span>
      </div>
    </div>
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
