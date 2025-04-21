use std::str::FromStr;

pub use base64::Engine;
pub use base64::engine::general_purpose;
use serde::Serialize;
use windows::{
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
    pub duration: u64,
    pub is_playing: bool,
    pub thumbnail: Option<String>,
    pub album: Option<String>,
    pub shuffle: Option<bool>,
    pub auto_repeat_mode: Option<AutoRepeatMode>,
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
        println!("Position: {}", position_ms);
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
        println!("Shuffle state: {}", shuffle_state);
        let new_state = !shuffle_state;
        let res = session.TryChangeShuffleActiveAsync(!shuffle_state)?.get()?;
        println!("Shuffle new state: {}, {}", new_state, res);
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

    pub fn get_progress(&self) -> Result<TrackProgress> {
        let session = self.get_current_session()?;
        let timeline = session.GetTimelineProperties()?;

        let position_timespan = timeline.Position()?;
        let duration_timespan = timeline.EndTime()?;

        // Get the duration value in 100-nanosecond units
        let position_ms = position_timespan.Duration / 10000; // Convert 100ns to ms
        let duration_ms = duration_timespan.Duration / 10000;

        Ok(TrackProgress {
            position: position_ms as u64,
            duration: duration_ms as u64,
        })
    }

    fn thumbnail(&self, session: Option<&Session>) -> Result<String> {
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

        // Encode the raw thumbnail bytes directly as base64
        // The thumbnail is likely already in a valid image format (JPEG/PNG)
        let encoder = general_purpose::STANDARD;
        let thumbnail_base64 = format!("data:image/jpeg;base64,{}", encoder.encode(&bytes));

        Ok(thumbnail_base64)
    }

    pub fn track_info(&self) -> Result<TrackInfo> {
        let session = self.get_current_session()?;

        let properties = session.TryGetMediaPropertiesAsync()?.get()?;
        let playback_info = session.GetPlaybackInfo()?;
        let playback_status = playback_info.PlaybackStatus()?;

        let is_playing = match playback_status {
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => true,
            _ => false,
        };

        let thumbnail = self.thumbnail(Some(&session)).ok();

        // Get track metadata
        let title = properties.Title()?.to_string();
        let album = properties.AlbumTitle().ok().map(|s| s.to_string());
        let artist = properties.Artist()?.to_string();

        let shuffle = self.get_shuffle_state(Some(&session)).ok();
        let auto_repeat_mode = self.get_auto_repeat_mode(Some(&session)).ok();

        let duration: std::time::Duration = session.GetTimelineProperties()?.EndTime()?.into();

        let track = TrackInfo {
            title,
            artist,
            album,
            shuffle,
            auto_repeat_mode,
            duration: duration.as_millis() as u64,
            thumbnail,
            is_playing,
        };

        Ok(track)
    }
}
