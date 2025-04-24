use std::str::FromStr;

use crate::utils;
use serde::Serialize;
use windows::{
    Foundation::TypedEventHandler,
    Media::{
        Control::{
            GlobalSystemMediaTransportControlsSession,
            GlobalSystemMediaTransportControlsSessionManager,
            GlobalSystemMediaTransportControlsSessionPlaybackStatus,
        },
        MediaPlaybackAutoRepeatMode,
    },
    Storage::Streams::{Buffer, DataReader, InputStreamOptions},
};

use GlobalSystemMediaTransportControlsSession as Session;
use GlobalSystemMediaTransportControlsSessionManager as SessionManager;

#[derive(Debug, Serialize, Clone)]
pub struct TrackProgress {
    pub position: u64,
    pub duration: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub thumbnail: Option<String>,
    pub album: Option<String>,
    pub duration: u64,
    pub accent_color: Option<u16>,
}
#[derive(Debug, Serialize, Clone)]
pub struct TrackControls {
    shuffle_enabled: bool,
    auto_repeat_mode_enabled: bool,
    next_enabled: bool,
    prev_enabled: bool,
    play_pause_enabled: bool,

    shuffle: bool,
    auto_repeat_mode: AutoRepeatMode,
    playing: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrackTimeline {
    progress: u64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AutoRepeatMode {
    None,
    Track,
    List,
}

impl From<MediaPlaybackAutoRepeatMode> for AutoRepeatMode {
    fn from(mode: MediaPlaybackAutoRepeatMode) -> Self {
        match mode {
            MediaPlaybackAutoRepeatMode::Track => AutoRepeatMode::Track,
            MediaPlaybackAutoRepeatMode::List => AutoRepeatMode::List,
            _ => AutoRepeatMode::None, // Default case
        }
    }
}

impl Into<MediaPlaybackAutoRepeatMode> for AutoRepeatMode {
    fn into(self) -> MediaPlaybackAutoRepeatMode {
        match self {
            AutoRepeatMode::Track => MediaPlaybackAutoRepeatMode::Track,
            AutoRepeatMode::List => MediaPlaybackAutoRepeatMode::List,
            AutoRepeatMode::None => MediaPlaybackAutoRepeatMode::None,
        }
    }
}

impl FromStr for AutoRepeatMode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(AutoRepeatMode::None),
            "track" => Ok(AutoRepeatMode::Track),
            "list" => Ok(AutoRepeatMode::List),
            _ => anyhow::bail!("Invalid auto-repeat mode"),
        }
    }
}

impl ToString for AutoRepeatMode {
    fn to_string(&self) -> String {
        match self {
            AutoRepeatMode::None => "none",
            AutoRepeatMode::Track => "track",
            AutoRepeatMode::List => "list",
        }
        .into()
    }
}

use anyhow::Result;

pub struct MediaManager {
    manager: SessionManager,
}

impl MediaManager {
    pub fn new() -> Result<Self> {
        let manager = SessionManager::RequestAsync()?.get()?;
        Ok(Self { manager })
    }

    pub fn get_current_session(&self) -> Result<Session> {
        let res = self.manager.GetCurrentSession()?;
        Ok(res)
    }

    pub fn toggle_play(&self) -> Result<bool> {
        let session = self.get_current_session()?;
        let res = session.TryTogglePlayPauseAsync()?.get()?;
        Ok(res)
    }

    pub fn next_track(&self) -> Result<bool> {
        let session = self.get_current_session()?;
        let res = session.TrySkipNextAsync()?.get()?;
        Ok(res)
    }

    pub fn previous_track(&self) -> Result<bool> {
        let current = self.get_current_session()?;
        let res = current.TrySkipPreviousAsync()?.get()?;
        Ok(res)
    }

    pub fn seek_to(&self, position_ms: u64) -> Result<bool> {
        let session = self.get_current_session()?;
        // Convert milliseconds to 100-nanosecond units
        let position_ns = position_ms as i64 * 10000;
        let res = session.TryChangePlaybackPositionAsync(position_ns)?.get()?;
        Ok(res)
    }

    pub fn get_shuffle_state(&self, session: Option<&Session>) -> Result<bool> {
        let session = match session {
            Some(s) => s,
            None => &self.get_current_session()?,
        };
        let playback_info = session.GetPlaybackInfo()?;
        let shuffle_state = playback_info.IsShuffleActive()?.Value()?;
        Ok(shuffle_state)
    }

    pub fn toggle_shuffle(&self) -> Result<()> {
        let session = self.get_current_session()?;
        let shuffle_state = self.get_shuffle_state(Some(&session))?;
        session.TryChangeShuffleActiveAsync(!shuffle_state)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_auto_repeat_mode(&self, session: Option<&Session>) -> Result<AutoRepeatMode> {
        let session = match session {
            Some(s) => s,
            None => &self.get_current_session()?,
        };
        let playback_info = session.GetPlaybackInfo()?;
        let repeat_state = playback_info.AutoRepeatMode()?.Value()?;
        let state = AutoRepeatMode::from(repeat_state);
        Ok(state)
    }

    pub fn set_auto_repeat_mode(&self, repeat_state: AutoRepeatMode) -> Result<()> {
        let session = self.get_current_session()?;
        let state: MediaPlaybackAutoRepeatMode = repeat_state.into();
        session.TryChangeAutoRepeatModeAsync(state)?.get()?;
        Ok(())
    }

    fn thumbnail(&self, session: Option<&Session>) -> Result<Vec<u8>> {
        let session = match session {
            Some(s) => s,
            None => &self.get_current_session()?,
        };
        let properties = session.TryGetMediaPropertiesAsync()?.get()?;

        // Process thumbnail
        let thumbnail = properties.Thumbnail()?;
        let buf = Buffer::Create(500_000)?;
        let stream = thumbnail
            .OpenReadAsync()?
            .get()?
            .ReadAsync(&buf, buf.Capacity()?, InputStreamOptions::ReadAhead)?
            .get()?;

        let byte_reader = DataReader::FromBuffer(&stream)?;
        let length = byte_reader.UnconsumedBufferLength()? as usize;
        let mut bytes = vec![0u8; length];
        byte_reader.ReadBytes(&mut bytes)?;

        Ok(bytes)
    }

    pub fn track_info(&self) -> Result<TrackInfo> {
        let session = self.get_current_session()?;

        let properties = session.TryGetMediaPropertiesAsync()?.get()?;
        let thumbnail_result = self.thumbnail(Some(&session));

        let mut thumbnail = None;
        let mut accent_color = None;

        if let Ok(thumbnail_bytes) = thumbnail_result {
            thumbnail = Some(utils::encode_image_to_base64(&thumbnail_bytes));
            match utils::extract_accent_color_hue(&thumbnail_bytes) {
                Ok(color) => accent_color = Some(color),
                Err(e) => {
                    tracing::error!("Failed to extract accent color: {}", e);
                }
            }
        }

        // Get track metadata
        let title = properties.Title()?.to_string();
        let album = properties.AlbumTitle().ok().map(|s| s.to_string());
        let artist = properties.Artist()?.to_string();

        let duration: std::time::Duration = session.GetTimelineProperties()?.EndTime()?.into();

        let track = TrackInfo {
            title,
            artist,
            thumbnail,
            album,
            accent_color,
            duration: duration.as_millis() as u64,
        };

        Ok(track)
    }

    pub fn remove_track_changed_handler(&self, token: i64) -> Result<()> {
        let session = self.get_current_session()?;
        session.RemoveMediaPropertiesChanged(token)?;
        Ok(())
    }

    pub fn track_changed<F>(&self, mut callback: F) -> Result<i64>
    where
        F: FnMut() -> () + Send + 'static,
    {
        let session = self.get_current_session()?;
        let handler = TypedEventHandler::new(move |_, _| {
            callback();
            windows::core::Result::Ok(())
        });

        let token = session.MediaPropertiesChanged(&handler)?;
        Ok(token)
    }

    pub fn track_controls(&self) -> Result<TrackControls> {
        let session = self.get_current_session()?;
        let playback_info = session.GetPlaybackInfo()?;

        let controls = playback_info.Controls()?;

        let shuffle_enabled = controls.IsShuffleEnabled()?;
        let auto_repeat_mode_enabled = controls.IsRepeatEnabled()?;
        let next_enabled = controls.IsNextEnabled()?;
        let prev_enabled = controls.IsPreviousEnabled()?;
        let play_pause_enabled = controls.IsPlayPauseToggleEnabled()?;

        let playing = match playback_info.PlaybackStatus()? {
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => true,
            _ => false,
        };

        let shuffle = match shuffle_enabled {
            true => self.get_shuffle_state(Some(&session))?,
            false => false,
        };

        let auto_repeat_mode = match auto_repeat_mode_enabled {
            true => self.get_auto_repeat_mode(Some(&session))?,
            false => AutoRepeatMode::None,
        };

        Ok(TrackControls {
            shuffle_enabled,
            auto_repeat_mode_enabled,
            next_enabled,
            prev_enabled,
            play_pause_enabled,

            shuffle,
            auto_repeat_mode,
            playing,
        })
    }
    pub fn track_controls_changed<F>(&self, mut callback: F) -> Result<i64>
    where
        F: FnMut() -> () + Send + 'static,
    {
        let session = self.get_current_session()?;

        let handler = TypedEventHandler::new(move |_, _| {
            callback();
            windows::core::Result::Ok(())
        });

        let token = session.PlaybackInfoChanged(&handler)?;
        Ok(token)
    }
    pub fn remove_track_controls_changed_handler(&self, token: i64) -> Result<()> {
        let session = self.get_current_session()?;
        session.RemovePlaybackInfoChanged(token)?;
        Ok(())
    }

    pub fn track_timeline(&self) -> Result<TrackTimeline> {
        let session = self.get_current_session()?;
        let timeline = session.GetTimelineProperties()?;
        let progress = timeline.Position()?;

        Ok(TrackTimeline {
            progress: progress.Duration as u64 / 10_000, // Convert 100ns to ms
        })
    }

    pub fn track_timeline_changed<F>(&self, mut callback: F) -> Result<i64>
    where
        F: FnMut() -> () + Send + 'static,
    {
        let session = self.get_current_session()?;

        let handler = TypedEventHandler::new(move |_, _| {
            callback();
            windows::core::Result::Ok(())
        });

        let token = session.TimelinePropertiesChanged(&handler)?;
        Ok(token)
    }

    pub fn remove_track_timeline_changed_handler(&self, token: i64) -> Result<()> {
        let session = self.get_current_session()?;
        session.RemoveTimelinePropertiesChanged(token)?;
        Ok(())
    }

    // The session change handler monitors for changes in the active media session
    pub fn session_changed<F>(&self, mut callback: F) -> Result<i64>
    where
        F: FnMut() -> () + Send + 'static,
    {
        let manager = &self.manager;

        let handler = TypedEventHandler::new(move |_, _| {
            callback();
            windows::core::Result::Ok(())
        });

        let token = manager.CurrentSessionChanged(&handler)?;
        Ok(token)
    }

    pub fn remove_session_changed_handler(&self, token: i64) -> Result<()> {
        let manager = &self.manager;
        manager.RemoveCurrentSessionChanged(token)?;
        Ok(())
    }

    // Gets a unique identifier for the current session to detect changes
    pub fn get_session_id(&self) -> Result<String> {
        if let Ok(session) = self.get_current_session() {
            // Use the source app user model ID as a unique identifier
            if let Ok(source_app_id) = session.SourceAppUserModelId() {
                return Ok(source_app_id.to_string());
            }
        }

        // Return a default if no session is available
        Ok("no_session".to_string())
    }
}
