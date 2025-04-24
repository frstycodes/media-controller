import { useEffect, useRef } from "react";

export function useHotKey(key: string, cb: () => void) {
  const refs = useRef({ key, cb });

  useEffect(() => {
    refs.current.cb = cb;
    refs.current.key = key;
  }, [cb, key]);

  function handleKeyDown(e: KeyboardEvent) {
    const { key, cb } = refs.current;
    if (e.key === key) {
      e.preventDefault();
      cb();
    }
  }
  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);
}
