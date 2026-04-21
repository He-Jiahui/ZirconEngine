use serde::{Deserialize, Serialize};

use super::{SoundClipId, SoundPlaybackId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundError {
    InvalidLocator { locator: String },
    BackendUnavailable { detail: String },
    NoProjectOpen,
    UnknownClip { clip: SoundClipId },
    UnknownPlayback { playback: SoundPlaybackId },
    InvalidMixRequest { frames: usize },
    Decode(String),
    Io(String),
}
