use serde::{Deserialize, Serialize};

use super::{
    SoundAutomationBindingId, SoundEffectId, SoundListenerId, SoundParameterId, SoundSourceId,
    SoundTrackId, SoundVolumeId,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundAutomationBinding {
    pub id: SoundAutomationBindingId,
    pub timeline_track_path: String,
    pub target: SoundAutomationTarget,
    pub parameter: SoundParameterId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundAutomationCurve {
    pub keyframes: Vec<SoundAutomationKeyframe>,
}

impl SoundAutomationCurve {
    pub fn from_keyframes(keyframes: impl Into<Vec<SoundAutomationKeyframe>>) -> Self {
        Self {
            keyframes: keyframes.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundAutomationKeyframe {
    pub time_seconds: f32,
    pub value: f32,
    pub interpolation: SoundAutomationInterpolation,
}

impl SoundAutomationKeyframe {
    pub fn linear(time_seconds: f32, value: f32) -> Self {
        Self {
            time_seconds,
            value,
            interpolation: SoundAutomationInterpolation::Linear,
        }
    }

    pub fn step(time_seconds: f32, value: f32) -> Self {
        Self {
            time_seconds,
            value,
            interpolation: SoundAutomationInterpolation::Step,
        }
    }

    pub fn smooth_step(time_seconds: f32, value: f32) -> Self {
        Self {
            time_seconds,
            value,
            interpolation: SoundAutomationInterpolation::SmoothStep,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundAutomationInterpolation {
    Step,
    Linear,
    SmoothStep,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundAutomationTarget {
    Track(SoundTrackId),
    Effect {
        track: SoundTrackId,
        effect: SoundEffectId,
    },
    Source(SoundSourceId),
    Listener(SoundListenerId),
    Volume(SoundVolumeId),
    SynthParameter(SoundParameterId),
}
