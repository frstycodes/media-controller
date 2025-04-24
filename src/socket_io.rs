use std::str::FromStr;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde::Deserialize;
use socketioxide::extract::{Data, SocketRef};

use crate::media_manager::{AutoRepeatMode, MediaManager};

const GET_MEDIA_DETAILS: &str = "get_media_details";
const TOGGLE_PLAY_PAUSE: &str = "toggle_play_pause";
const NEXT_TRACK: &str = "next_track";
const PREVIOUS_TRACK: &str = "previous_track";
const SEEK: &str = "seek";
const SET_REPEAT_MODE: &str = "set_repeat_mode";
const TOGGLE_SHUFFLE: &str = "toggle_shuffle";

const TRACK_INFO: &str = "track_info";
const TRACK_CONTROLS: &str = "track_controls";
const TRACK_TIMELINE: &str = "track_timeline";

#[derive(Debug, Deserialize)]
pub struct SeekPosition {
    /// Position in milliseconds
    pub position: u64,
}

struct HandlerSession {
    media_manager: Arc<Mutex<MediaManager>>,
    track_changed_token: Option<i64>,
    track_controls_token: Option<i64>,
    track_timeline_token: Option<i64>,
}

impl HandlerSession {
    fn new(media_manager: MediaManager) -> Self {
        Self {
            media_manager: Arc::new(Mutex::new(media_manager)),
            track_changed_token: None,
            track_controls_token: None,
            track_timeline_token: None,
        }
    }

    fn restart_listeners(&mut self, socket: SocketRef) {
        tracing::info!("Restarting listeners");
        self.cleanup();
        self.setup_listeners(socket);
    }

    fn emit_intial_data(&self, socket: SocketRef) {
        let mm = &self.media_manager;
        emit_track_info(&mm, &socket).ok();
        emit_track_controls(&mm, &socket).ok();
        emit_track_timeline(&mm, &socket).ok();
    }

    fn setup_listeners(&mut self, socket: SocketRef) {
        if let Ok(token) =
            on_track_controls_changed(Arc::clone(&self.media_manager), socket.clone())
        {
            self.track_controls_token = Some(token);
        }

        if let Ok(token) =
            on_track_timeline_changed(Arc::clone(&self.media_manager), socket.clone())
        {
            self.track_timeline_token = Some(token);
        }

        if let Ok(token) = on_track_changed(Arc::clone(&self.media_manager), socket.clone()) {
            self.track_changed_token = Some(token);
        }
    }

    fn cleanup(&mut self) {
        tracing::info!("Cleaning up handler session");
        if let Ok(manager) = self.media_manager.lock() {
            if let Some(token) = self.track_changed_token.take() {
                if let Err(e) = manager.remove_track_changed_handler(token) {
                    tracing::error!("Failed to unregister track changed callback: {}", e);
                }
            }

            if let Some(token) = self.track_controls_token.take() {
                if let Err(e) = manager.remove_track_controls_changed_handler(token) {
                    tracing::error!("Failed to unregister track controls callback: {}", e);
                }
            }

            if let Some(token) = self.track_timeline_token.take() {
                if let Err(e) = manager.remove_track_timeline_changed_handler(token) {
                    tracing::error!("Failed to unregister track timeline callback: {}", e);
                }
            }
        } else {
            tracing::error!("Failed to lock media manager for cleanup");
        }
    }
}

pub fn on_connect(socket: SocketRef) {
    tracing::info!("socket connected: {}", socket.id);

    let mut session = HandlerSession::new(MediaManager::new().unwrap());
    let media_manager = Arc::clone(&session.media_manager);

    session.emit_intial_data(socket.clone());
    session.setup_listeners(socket.clone());

    let mm_details = Arc::clone(&media_manager);
    socket.on(GET_MEDIA_DETAILS, move |socket: SocketRef| {
        tracing::info!("Getting media details");
        let media_manager = Arc::clone(&mm_details);
        let socket = socket.clone();
        if let Err(e) = emit_track_info(&media_manager, &socket) {
            tracing::error!("Failed to get media details: {}", e);
        }
    });

    // HANDLE PLAY/PAUSE TOGGLE
    let mm_play_pause = Arc::clone(&media_manager);
    socket.on(TOGGLE_PLAY_PAUSE, move |_: SocketRef| {
        let media_manager = Arc::clone(&mm_play_pause);
        if let Ok(manager) = media_manager.lock() {
            if let Err(e) = manager.toggle_play() {
                tracing::error!("Failed to toggle play/pause: {}", e);
            }
        }
    });

    // HANDLE NEXT TRACK
    let mm_next = Arc::clone(&media_manager);
    socket.on(NEXT_TRACK, move |_: SocketRef| {
        if let Ok(manager) = mm_next.lock() {
            if let Err(e) = manager.next_track() {
                tracing::error!("Failed to skip to next track: {}", e);
            }
        }
    });

    // HANDLE PREVIOUS TRACK
    let mm_prev = Arc::clone(&media_manager);
    socket.on(PREVIOUS_TRACK, move |_: SocketRef| {
        if let Ok(manager) = mm_prev.lock() {
            if let Err(e) = manager.previous_track() {
                tracing::error!("Failed to go to previous track: {}", e);
            }
        }
    });

    // HANDLE REPEAT MODE
    let mm_set_repeat_mode = Arc::clone(&media_manager);
    socket.on(SET_REPEAT_MODE, move |_: SocketRef, data: Data<String>| {
        tracing::info!("Setting auto repeat mode: {:?}", data.0);
        if let Ok(mode) = AutoRepeatMode::from_str(&data) {
            if let Ok(manager) = mm_set_repeat_mode.lock() {
                if let Err(e) = manager.set_auto_repeat_mode(mode) {
                    tracing::error!("Failed to set auto repeat mode: {}", e);
                }
            }
        } else {
            tracing::error!("Invalid auto repeat mode: {}", data.0);
        }
    });

    // TOGGLE SHUFFLE
    let mm_toggle_shuffle = Arc::clone(&media_manager);
    socket.on(TOGGLE_SHUFFLE, move |_: SocketRef| {
        let mm = Arc::clone(&mm_toggle_shuffle);
        tokio::spawn(async move {
            if let Ok(manager) = mm.lock() {
                if let Err(e) = manager.toggle_shuffle() {
                    tracing::error!("Failed to toggle shuffle: {}", e);
                }
            }
        });
    });

    // HANDLE SEEK
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

    // SET UP EVENT LISTENERS AND STORE THEIR TOKENS IN THE SESSION

    let session_arc = Arc::new(Mutex::new(session));
    let socket_id = socket.id.clone();

    let session_for_change = Arc::clone(&session_arc);
    let socket_clone = socket.clone();
    let callback = move || {
        let session = Arc::clone(&session_for_change);
        let socket = socket_clone.clone();
        std::thread::spawn(move || {
            if let Ok(mut session) = session.lock() {
                session.restart_listeners(socket.clone());
                session.emit_intial_data(socket);
            } else {
                tracing::error!("Failed to lock session for restart on change");
            }
        });
    };

    let session_change_token = if let Ok(manager) = media_manager.lock() {
        match manager.session_changed(callback) {
            Ok(token) => Some(token),
            Err(e) => {
                tracing::error!("Failed to register session change callback: {}", e);
                None
            }
        }
    } else {
        tracing::error!("Failed to lock media manager for session change callback");
        None
    };

    let session_for_dc = Arc::clone(&session_arc);
    let disconnect_handler = move || {
        tracing::info!("socket disconnected: {}", socket_id);

        if let Ok(mut session) = session_for_dc.lock() {
            session.cleanup();
            if let Ok(manager) = session.media_manager.lock() {
                if let Some(token) = session_change_token {
                    manager.remove_session_changed_handler(token).ok();
                }
            }
        } else {
            tracing::error!("Failed to lock session for cleanup on disconnect");
        }
    };

    socket.on_disconnect(disconnect_handler);
}

fn on_track_changed(media_manager: Arc<Mutex<MediaManager>>, socket: SocketRef) -> Result<i64> {
    let mm_handler = Arc::clone(&media_manager);
    let socket_clone = socket.clone();

    let callback = move || {
        let mm = Arc::clone(&mm_handler);
        let socket = socket_clone.clone();
        tracing::info!("Track changed");

        if let Err(e) = emit_track_info(&mm, &socket) {
            tracing::error!("Failed to get track info: {}", e);
        }
    };

    let token = match media_manager.lock() {
        Ok(manager) => manager.track_changed(callback),
        Err(e) => Err(anyhow::anyhow!("Failed to lock media manager: {}", e)),
    }?;

    tracing::info!("Registered track change callback with token: {}", token);
    Ok(token)
}

fn on_track_controls_changed(
    media_manager: Arc<Mutex<MediaManager>>,
    socket: SocketRef,
) -> Result<i64> {
    let mm_handler = Arc::clone(&media_manager);
    let socket_clone = socket.clone();

    let callback = move || {
        let mm = Arc::clone(&mm_handler);
        let socket = socket_clone.clone();
        tracing::info!("Track Controls changed");

        // std::thread::spawn(move || {
        if let Err(e) = emit_track_controls(&mm, &socket) {
            tracing::error!("Failed to get track controls info: {}", e);
        }
        // });
    };

    let token = match media_manager.lock() {
        Ok(manager) => manager.track_controls_changed(callback),
        Err(e) => Err(anyhow::anyhow!("Failed to lock media manager: {}", e)),
    }?;

    tracing::info!(
        "Registered Playback Info change callback with token: {}",
        token
    );
    Ok(token)
}

fn on_track_timeline_changed(
    media_manager: Arc<Mutex<MediaManager>>,
    socket: SocketRef,
) -> Result<i64> {
    let mm_handler = Arc::clone(&media_manager);
    let socket_clone = socket.clone();

    let callback = move || {
        // let mm = Arc::clone(&mm_handler);
        // let socket = socket_clone.clone();
        tracing::info!("Track timeline changed");

        // std::thread::spawn(move || {
        if let Err(e) = emit_track_timeline(&mm_handler, &socket_clone) {
            tracing::error!("Failed to get track timeline info: {}", e);
        }
        // });
    };

    let token = match media_manager.lock() {
        Ok(manager) => manager.track_timeline_changed(callback),
        Err(e) => Err(anyhow::anyhow!("Failed to lock media manager: {}", e)),
    }?;

    tracing::info!(
        "Registered Track Timeline change callback with token: {}",
        token
    );
    Ok(token)
}

fn emit_track_info(media_manager: &Arc<Mutex<MediaManager>>, socket: &SocketRef) -> Result<()> {
    if let Ok(manager) = media_manager.lock() {
        if let Ok(track) = manager.track_info() {
            drop(manager);
            if let Err(e) = socket.emit(TRACK_INFO, &track) {
                tracing::error!("Failed to emit track info: {}", e);
            }
            return Ok(());
        }
    }
    anyhow::bail!("Failed to get track info");
}

fn emit_track_controls(media_manager: &Arc<Mutex<MediaManager>>, socket: &SocketRef) -> Result<()> {
    if let Ok(manager) = media_manager.lock() {
        if let Ok(controls) = manager.track_controls() {
            drop(manager);
            if let Err(e) = socket.emit(TRACK_CONTROLS, &controls) {
                tracing::error!("Failed to emit track controls: {}", e);
            }
            return Ok(());
        }
    }

    anyhow::bail!("Failed to get track controls")
}

fn emit_track_timeline(media_manager: &Arc<Mutex<MediaManager>>, socket: &SocketRef) -> Result<()> {
    if let Ok(manager) = media_manager.lock() {
        if let Ok(controls) = manager.track_timeline() {
            drop(manager);
            if let Err(e) = socket.emit(TRACK_TIMELINE, &controls) {
                tracing::error!("Failed to emit timeline controls: {}", e);
            }
            return Ok(());
        }
    }
    anyhow::bail!("Failed to get timeline controls");
}
