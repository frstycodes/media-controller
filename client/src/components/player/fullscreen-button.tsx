import { Fullscreen } from "lucide-react";
import { Button } from "../ui/button";

export function FullscreenButton() {
  const toggleFullscreen = () => {
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      document.documentElement.requestFullscreen();
    }
  };

  return (
    <Button
      onClick={toggleFullscreen}
      className="fixed z-10 bottom-0 right-0 text-muted-foreground"
      variant="ghost"
    >
      <Fullscreen />
    </Button>
  );
}
