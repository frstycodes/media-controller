import { useEffect, useState } from "react";

const INTERACTION_EVENTS = [
  "mousedown",
  "mousemove",
  "keydown",
  "touchstart",
  "scroll",
] as const;

export function useInactivityTracker(timeout: number): boolean {
  const [active, setActive] = useState(true);

  useEffect(() => {
    const controller = new AbortController();
    let inactivityTimer: number | undefined;

    const resetTimer = () => {
      clearTimeout(inactivityTimer);
      setActive(true);
      inactivityTimer = setTimeout(() => {
        setActive(false);
      }, timeout);
    };

    resetTimer();

    for (const ev of INTERACTION_EVENTS) {
      document.addEventListener(ev, resetTimer, { signal: controller.signal });
    }

    return () => {
      clearTimeout(inactivityTimer);
      controller.abort();
    };
  }, [timeout]);

  return active;
}
