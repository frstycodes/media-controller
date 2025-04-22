type TrackThumbnailProps = {
  thumbnail: string;
};

export function TrackThumbnail(props: TrackThumbnailProps) {
  return (
    <div className="relative grow sm:grow-0 w-full sm:w-[unset]">
      {[0, 1].map((i) => (
        <img
          data-blur={i == 1 ? "true" : "false"}
          key={i}
          src={props.thumbnail}
          alt="thumbnail"
          className="w-full data-[blur=true]:blur-[100px] data-[blur=true]:animate-pulse data-[blur=true]:absolute data-[blur=true]:inset-0 data-[blur=true]:-z-10 aspect-square sm:w-80  rounded-lg z-10 border-3 border-white/5"
        />
      ))}
    </div>
  );
}
