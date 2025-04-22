import "./App.css";
import { useEffect, useRef, useState } from "react";
import { AnimatePresence } from "motion/react";
import { IO, TrackInfo, events } from "./lib/socket-io";
import { useInactivityTracker } from "./hooks/use-inactivity-tracker";
import { PlayerControls } from "./components/player/player-controls";
import { TrackMetaData } from "./components/player/track-display";
import { ProgressBar } from "./components/player/progress-bar";
import { FullscreenButton } from "./components/player/fullscreen-button";
import { TrackThumbnail } from "./components/player/track-thumbnail";

const SOCKET_URL = "http://192.168.0.105:3001/ws";
const INACTIVITY_TIMEOUT = 10 * 1000;

function App() {
  const [track, setTrack] = useState<TrackInfo | null>(null);
  const active = useInactivityTracker(INACTIVITY_TIMEOUT);
  const io = useRef<IO>(null!);

  useEffect(() => {
    io.current = new IO(SOCKET_URL);
    const socket = io.current.getSocket();

    socket.emit(events.GET_MEDIA_DETAILS);
    socket.on(events.MEDIA_DETAILS, (track: TrackInfo) => {
      const root = document.documentElement;
      root.style.setProperty("--primary-hue", track.accent_color);
      setTrack(track);
    });

    return () => {
      socket.off(events.MEDIA_DETAILS);
      socket.disconnect();
    };
  }, []);

  if (!track)
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-6">
        <div className="text-center">
          <div className="mb-4 flex justify-center">
            <div className="animate-spin rounded-full border-t-2 border-b-2 border-gray-600 h-10 w-10"></div>
          </div>
          <h2 className="mb-2 text-lg font-bold">Waiting for media...</h2>
          <p className="text-gray-600 dark:text-gray-300 text-sm font-medium">
            No track is currently playing.
          </p>
        </div>
      </div>
    );

  return (
    <div className="flex-1 flex-col sm:flex-row flex justify-center items-center">
      {active && <FullscreenButton />}
      <img
        src={track.thumbnail}
        className="fixed h-screen w-screen inset-0 opacity-20 blur-[100px] object-cover"
      />
      <div className="flex flex-col sm:flex-row items-center relative gap-8 w-full sm:max-w-[800px]">
        <TrackThumbnail thumbnail={track.thumbnail} />
        <div className="flex-1 flex w-full flex-col justify-center space-y-5">
          <TrackMetaData track={track} active={active} />
          <div>
            <AnimatePresence mode="popLayout">
              <ProgressBar io={io.current} active={active} />
              {active && <PlayerControls track={track} io={io.current} />}
            </AnimatePresence>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
