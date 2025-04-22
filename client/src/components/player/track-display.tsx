import { Disc3 } from "lucide-react";
import { motion } from "motion/react";
import { TrackInfo } from "@/lib/socket-io";
import { cn } from "@/lib/utils";

type TrackDisplayProps = {
  track: TrackInfo;
  active: boolean;
};

export function TrackMetaData({ track, active }: TrackDisplayProps) {
  return (
    <motion.div layout="position" className="overflow-hidden">
      <p className="text-medium items-center text-shadow-md text-left text-muted-foreground">
        {track.artist}
      </p>
      <h1
        className={cn(
          "font-extrabold transition-all text-left overflow-hidden text-2xl sm:text-4xl text-shadow-lg",
          !active && "text-3xl sm:text-5xl",
        )}
      >
        {track.title}
      </h1>
      {!!track.album && (
        <p className="text-medium flex gap-1 pt-2 text-sm items-center text-shadow-md text-left text-muted-foreground">
          <Disc3 className="size-4 animate-spin" />
          {track.album}
        </p>
      )}
    </motion.div>
  );
}
