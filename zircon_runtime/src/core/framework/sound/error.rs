use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    ExternalAudioSourceHandle, SoundAutomationBindingId, SoundClipId, SoundEffectId,
    SoundImpulseResponseId, SoundListenerId, SoundParameterId, SoundPlaybackId, SoundSourceId,
    SoundTimelineSequenceId, SoundTrackId, SoundVolumeId,
};

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundError {
    #[error("invalid sound locator `{locator}`")]
    InvalidLocator { locator: String },
    #[error("sound backend unavailable: {detail}")]
    BackendUnavailable { detail: String },
    #[error("no project is open for sound asset resolution")]
    NoProjectOpen,
    #[error("unknown sound clip {clip:?}")]
    UnknownClip { clip: SoundClipId },
    #[error("unknown sound playback {playback:?}")]
    UnknownPlayback { playback: SoundPlaybackId },
    #[error("unknown track {track:?}")]
    UnknownTrack { track: SoundTrackId },
    #[error("unknown effect {effect:?}")]
    UnknownEffect { effect: SoundEffectId },
    #[error("unknown impulse response {impulse_response:?}")]
    UnknownImpulseResponse {
        impulse_response: SoundImpulseResponseId,
    },
    #[error("unknown HRTF profile `{profile_id}`")]
    UnknownHrtfProfile { profile_id: String },
    #[error("unknown automation binding {binding:?}")]
    UnknownAutomationBinding { binding: SoundAutomationBindingId },
    #[error("unknown timeline sequence {sequence:?}")]
    UnknownTimelineSequence { sequence: SoundTimelineSequenceId },
    #[error("unknown dynamic sound event `{event_id}`")]
    UnknownDynamicEvent { event_id: String },
    #[error("unknown dynamic sound event handler `{plugin_id}:{handler_id}`")]
    UnknownDynamicEventHandler {
        plugin_id: String,
        handler_id: String,
    },
    #[error("unknown send from track {track:?} to {target:?}")]
    UnknownSend {
        track: SoundTrackId,
        target: SoundTrackId,
    },
    #[error("unknown source {source_id:?}")]
    UnknownSource { source_id: SoundSourceId },
    #[error("unknown external audio source {handle:?}")]
    UnknownExternalSource { handle: ExternalAudioSourceHandle },
    #[error("unknown listener {listener:?}")]
    UnknownListener { listener: SoundListenerId },
    #[error("unknown volume {volume:?}")]
    UnknownVolume { volume: SoundVolumeId },
    #[error("unknown sound parameter {parameter:?}")]
    UnknownParameter { parameter: SoundParameterId },
    #[error("invalid mix request for {frames} frames")]
    InvalidMixRequest { frames: usize },
    #[error("invalid mixer graph: {0}")]
    InvalidMixerGraph(String),
    #[error("invalid sound effect: {0}")]
    InvalidEffect(String),
    #[error("invalid sound parameter: {0}")]
    InvalidParameter(String),
    #[error("unsupported advanced sound feature: {0}")]
    UnsupportedAdvancedFeature(String),
    #[error("sound decode failed: {0}")]
    Decode(String),
    #[error("sound io failed: {0}")]
    Io(String),
}
