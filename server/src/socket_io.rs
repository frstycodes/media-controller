use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use socketioxide::extract::{Data, SocketRef};
use tokio::time::sleep;

use crate::media_manager::{AutoRepeatMode, MediaManager};

const GET_MEDIA_DETAILS: &str = "get_media_details";
const TOGGLE_PLAY_PAUSE: &str = "toggle_play_pause";
const NEXT_TRACK: &str = "next_track";
const PREVIOUS_TRACK: &str = "previous_track";
const SEEK: &str = "seek";
const SET_REPEAT_MODE: &str = "set_repeat_mode";
const TOGGLE_SHUFFLE: &str = "toggle_shuffle";

const TRACK_PROGRESS: &str = "track_progress";
const MEDIA_DETAILS: &str = "media_details";

#[derive(Debug, Deserialize)]
pub struct SeekPosition {
    /// Position in milliseconds
    pub position: u64,
}

pub fn on_connect(socket: SocketRef, io: SocketRef) {
    tracing::info!("socket connected: {}", socket.id);
    let media_manager = Arc::new(Mutex::new(MediaManager::new().unwrap()));

    // Create a clone for the get_media_details handler
    let mm_details = Arc::clone(&media_manager);
    socket.on(GET_MEDIA_DETAILS, move |socket: SocketRef| {
        tracing::info!("Getting media details");

        let media_manager = Arc::clone(&mm_details);
        let socket = socket.clone();

        tokio::spawn(async move {
            if let Err(e) = get_and_emit_track_info(&media_manager, &socket).await {
                tracing::error!("Failed to get media details: {}", e);
            }
        });
    });

    // Handle play/pause toggle
    let mm_play_pause = Arc::clone(&media_manager);
    let io_play_pause = io.clone();
    socket.on(TOGGLE_PLAY_PAUSE, move |_socket: SocketRef| {
        let media_manager = Arc::clone(&mm_play_pause);
        let io = io_play_pause.clone();
        tokio::spawn(async move {
            if let Ok(manager) = media_manager.lock() {
                if let Err(e) = manager.toggle_play() {
                    tracing::error!("Failed to toggle play/pause: {}", e);
                } else {
                    // Emit updated media status after toggling
                    let mm = Arc::clone(&media_manager);
                    let io = io.clone();
                    tokio::spawn(async move {
                        sleep(Duration::from_millis(100)).await;
                        get_and_emit_track_info(&mm, &io).await.ok();
                    });
                }
            }
        });
    });

    // Handle next track
    let mm_next = Arc::clone(&media_manager);
    let io_next = io.clone();
    socket.on(NEXT_TRACK, move |_: SocketRef| {
        let media_manager = Arc::clone(&mm_next);
        let io = io_next.clone();
        tokio::spawn(async move {
            if let Ok(manager) = media_manager.lock() {
                if let Err(e) = manager.next_track() {
                    tracing::error!("Failed to skip to next track: {}", e);
                } else {
                    // Emit updated media status after toggling
                    let mm = Arc::clone(&media_manager);
                    let io = io.clone();
                    tokio::spawn(async move {
                        sleep(Duration::from_millis(200)).await;
                        get_and_emit_track_info(&mm, &io).await.ok();
                    });
                }
            }
        });
    });

    // Handle previous track
    let mm_prev = Arc::clone(&media_manager);
    let io_prev = io.clone();
    socket.on(PREVIOUS_TRACK, move |_: SocketRef| {
        let mm = Arc::clone(&mm_prev);
        let io = io_prev.clone();
        tokio::spawn(async move {
            if let Ok(manager) = mm.lock() {
                if let Err(e) = manager.previous_track() {
                    tracing::error!("Failed to go to previous track: {}", e);
                } else {
                    let mm = Arc::clone(&mm);
                    let io = io.clone();
                    tokio::spawn(async move {
                        sleep(Duration::from_millis(200)).await;
                        get_and_emit_track_info(&mm, &io).await.ok();
                    });
                }
            }
        });
    });

    let mm_set_repeat_mode = Arc::clone(&media_manager);
    let io_set_repeat_mode = io.clone();
    socket.on(SET_REPEAT_MODE, move |_: SocketRef, data: Data<String>| {
        let mm = Arc::clone(&mm_set_repeat_mode);
        let io = io_set_repeat_mode.clone();
        let mode_res = AutoRepeatMode::from_str(&data.0);
        if mode_res.is_err() {
            tracing::error!("Invalid auto repeat mode: {}", data.0);
            return;
        }
        let mode = mode_res.unwrap();
        tracing::info!("Setting auto repeat mode: {:?}", mode);
        tokio::spawn(async move {
            if let Ok(manager) = mm.lock() {
                if let Err(e) = manager.set_auto_repeat_mode(mode) {
                    tracing::error!("Failed to set auto repeat mode: {}", e);
                } else {
                    let mm = Arc::clone(&mm);
                    let io = io.clone();
                    tokio::spawn(async move {
                        sleep(Duration::from_millis(200)).await;
                        get_and_emit_track_info(&mm, &io).await.ok();
                    });
                }
            }
        });
    });

    // TOGGLE SHUFFLE
    let mm_toggle_shuffle = Arc::clone(&media_manager);
    let io_toggle_shuffle = io.clone();
    socket.on(TOGGLE_SHUFFLE, move |_: SocketRef| {
        let mm = Arc::clone(&mm_toggle_shuffle);
        let io = io_toggle_shuffle.clone();
        tokio::spawn(async move {
            if let Ok(manager) = mm.lock() {
                if let Err(e) = manager.toggle_shuffle() {
                    tracing::error!("Failed to toggle shuffle: {}", e);
                } else {
                    tracing::info!("Toggling shuffle");
                    let mm = Arc::clone(&mm);
                    let io = io.clone();
                    tokio::spawn(async move {
                        sleep(Duration::from_millis(200)).await;
                        get_and_emit_track_info(&mm, &io).await.ok();
                    });
                }
            }
        });
    });

    // Handle seek
    let mm_seek = Arc::clone(&media_manager);
    socket.on(SEEK, move |_socket: SocketRef, data: Data<SeekPosition>| {
        let mm = Arc::clone(&mm_seek);
        let position = data.position;
        tokio::spawn(async move {
            if let Ok(manager) = mm.lock() {
                if let Err(e) = manager.seek_to(position) {
                    tracing::error!("Failed to seek to position {}: {}", position, e);
                }
            }
        });
    });

    // Set up media | playing_status change detector
    let mm_change = Arc::clone(&media_manager);
    let io_change = io.clone();
    tokio::spawn(async move {
        let mut current_title = String::new();
        let mut current_artist = String::new();
        let mut current_playing = false;

        loop {
            if let Ok(manager) = mm_change.lock() {
                if let Ok(track) = manager.track_info() {
                    if track.title != current_title
                        || track.artist != current_artist
                        || track.is_playing != current_playing
                    {
                        current_title = track.title.clone();
                        current_artist = track.artist.clone();
                        current_playing = track.is_playing;

                        let _ = io_change.emit(MEDIA_DETAILS, &track);
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });

    // Set up progress tracking at regular intervals
    let mm_progress = Arc::clone(&media_manager);
    let io_progress = io.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(manager) = mm_progress.lock() {
                if let Ok(progress) = manager.get_progress() {
                    let _ = io_progress.emit(TRACK_PROGRESS, &progress);
                }
            }
            sleep(Duration::from_millis(1000)).await;
        }
    });
}

async fn get_and_emit_track_info(
    media_manager: &Arc<Mutex<MediaManager>>,
    socket: &SocketRef,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(manager) = media_manager.lock() {
        if let Ok(track) = manager.track_info() {
            if let Err(e) = socket.emit(MEDIA_DETAILS, &track) {
                tracing::error!("Failed to emit track info: {}", e);
            }
            return Ok(());
        }
    }
    Err("Failed to get track info".into())
}
