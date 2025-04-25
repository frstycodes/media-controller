# Media Broadcaster

Media Broadcaster is a Rust windows service which actively broadcasts currently playing media details and handles to control the media over local network using Socket IO which can then be used to build media visualizers/controllers or any other application which can consume the data.


## Events
Events are emitted from SocketIO Server to the client with event code and payload.

1. __Track Info__
    - Current Track/Media Details.
    - Code: `track_info`
    - Payload:
      ```ts
      type TrackInfo =  {
        title: string,
        artist: string,
        album: string | null,
        duration: number, // In Miliseconds
        thumbnail: string, // Base64 encoded thumbnail image
        accent_color: number, // Only Hue 0-360
      }
      ```

2. __Track Controls Data__
   - Track Controls data.
   - Code: `track_controls`
   - Payload:
     ```ts
     type TrackControls = {
        shuffle_enabled: boolean;
        auto_repeat_mode_enabled: boolean;
        next_enabled: boolean;
        prev_enabled: boolean;
        play_pause_enabled: boolean;

        shuffle: boolean;
        auto_repeat_mode: "none" | "track" | "list";
        playing: boolean;
     }
     ```

3. __Track Timeline__
   - Track timeline data.
   - Code: `track_timeline`
   - Payload:
     ```ts
     type TrackTimeline = {
       progress: number; // In Miliseconds
     }
      ```

## Functions
Functions are events emitted from SocketIO Client to control/request data from the service.

1. __Toggle Play/Pause__
   - Play or Pause the current track.
   - Code: `toggle_play_pause`
   - Payload: `null`

2. __Next Track__
   - Play the next track in the queue.
   - Code: `next_track`
   - Payload: `null`

3. __Previous Track__
    - Play the previous track in the queue.
    - Code: `prev_track`
    - Payload: `null`

4. __Seek Track__
    - Seek the current track to the given time in milliseconds.
    - Code: `seek_track`
    - Payload:
      ```ts
      type SeekPayload = {
        position: number; // In Miliseconds
      }
      ```

5. __Set Repeat Mode__
    - Set the repeat mode for the current track.
    - Code: `set_repeat_mode`
    - Payload: `"none" | "track" | "list"`

6. __Toggle Shuffle__
    - Toggles the shuffle mode for the current track.
    - Code: `toggle_shuffle`
    - Payload: `null`


## Installation
1. Download the latest version of media-controller.exe and client.zip from [Releases](https://github.com/frstycodes/media-controller/releases)
2. Place media-controller.exe in a folder and unzip client.zip in the same folder.
3. Open a terminal in the folder and run:
    ```bash
    ./media-controller.exe -f
    ```
    This will start the service and print the urls for the web client and socket io server.



For details on arguments, run the command with '-h' or '--help' flag:
```bash
./media-controller.exe -h
```


## Bring in your own Client
You can build your own client using the SocketIO server and the events emitted from it. The client can be built using any framework or library which supports SocketIO. And you can use the flag `-d` or `--frontend-directory` to specify the directory of your client.
```bash
./media-controller.exe -f -d <path-to-your-client>
```


## Limitations
- Currently only supports Windows OS.
- Timeline Position updates every 4 to 5 seconds. I don't know if it's a limitation of the Windows API itself or the way I'm using it. I haven't really looked into it.
