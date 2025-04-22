import { io, Socket } from "socket.io-client";

export const events = {
  GET_MEDIA_DETAILS: "get_media_details",
  TOGGLE_PLAY_PAUSE: "toggle_play_pause",
  NEXT_TRACK: "next_track",
  PREVIOUS_TRACK: "previous_track",
  SEEK: "seek",
  TOGGLE_SHUFFLE: "toggle_shuffle",
  SET_REPEAT_MODE: "set_repeat_mode",
  MEDIA_CHANGED: "media_changed",
  TRACK_PROGRESS: "track_progress",
  MEDIA_DETAILS: "media_details",
};

export type TrackInfo = {
  title: string;
  artist: string;
  album: string | null;
  duration: number;
  thumbnail: string;
  is_playing: boolean;
  shuffle: boolean;
  auto_repeat_mode: AutoRepeatMode;
  accent_color: string; // only hue
};

export type TrackProgress = {
  position: number;
  duration: number;
};

export enum AutoRepeatMode {
  None = "none",
  Track = "track",
  List = "list",
}

export class IO {
  public socket: Socket;

  constructor(url: string) {
    this.socket = io(url);
  }

  public getSocket() {
    return this.socket;
  }

  public setSocket(socket: Socket) {
    this.socket = socket;
  }

  togglePlayPause() {
    this.socket.emit(events.TOGGLE_PLAY_PAUSE);
  }

  nextTrack() {
    this.socket.emit(events.NEXT_TRACK);
  }

  previousTrack() {
    this.socket.emit(events.PREVIOUS_TRACK);
  }

  toggleShuffle() {
    this.socket.emit(events.TOGGLE_SHUFFLE);
  }

  setRepeatMode(mode: AutoRepeatMode) {
    this.socket.emit(events.SET_REPEAT_MODE, mode);
  }

  seek(position: number) {
    this.socket.emit(events.SEEK, { position });
  }

  getMediaDetails() {
    this.socket.emit(events.GET_MEDIA_DETAILS);
  }
}
