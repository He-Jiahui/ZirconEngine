use serde::{Deserialize, Serialize};

use super::{
    SoundAutomationBinding, SoundDynamicEventCatalog, SoundEffectDescriptor, SoundSourceDescriptor,
    SoundTrackId,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundMixerGraph {
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub tracks: Vec<SoundTrackDescriptor>,
    pub sources: Vec<SoundSourceDescriptor>,
    pub automation_bindings: Vec<SoundAutomationBinding>,
    pub dynamic_events: SoundDynamicEventCatalog,
}

impl SoundMixerGraph {
    pub fn default_stereo(sample_rate_hz: u32) -> Self {
        Self {
            sample_rate_hz,
            channel_count: 2,
            tracks: vec![SoundTrackDescriptor::master()],
            sources: Vec::new(),
            automation_bindings: Vec::new(),
            dynamic_events: SoundDynamicEventCatalog::empty(),
        }
    }

    pub fn master_track(&self) -> Option<&SoundTrackDescriptor> {
        self.tracks
            .iter()
            .find(|track| track.id == SoundTrackId::master())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundMixerSnapshot {
    pub graph: SoundMixerGraph,
    pub meters: Vec<SoundTrackMeter>,
    pub latency_frames: usize,
    pub ray_tracing: SoundRayTracingConvolutionStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundTrackDescriptor {
    pub id: SoundTrackId,
    pub display_name: String,
    pub parent: Option<SoundTrackId>,
    pub sends: Vec<SoundTrackSend>,
    pub effects: Vec<SoundEffectDescriptor>,
    pub controls: SoundTrackControls,
}

impl SoundTrackDescriptor {
    pub fn master() -> Self {
        Self {
            id: SoundTrackId::master(),
            display_name: "Master".to_string(),
            parent: None,
            sends: Vec::new(),
            effects: Vec::new(),
            controls: SoundTrackControls::default(),
        }
    }

    pub fn child(id: SoundTrackId, display_name: impl Into<String>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            parent: Some(SoundTrackId::master()),
            sends: Vec::new(),
            effects: Vec::new(),
            controls: SoundTrackControls::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundTrackControls {
    pub gain: f32,
    pub pan: f32,
    pub left_gain: f32,
    pub right_gain: f32,
    pub delay_frames: usize,
    pub invert_left_phase: bool,
    pub invert_right_phase: bool,
    pub mute: bool,
    pub solo: bool,
    pub bypass_effects: bool,
}

impl Default for SoundTrackControls {
    fn default() -> Self {
        Self {
            gain: 1.0,
            pan: 0.0,
            left_gain: 1.0,
            right_gain: 1.0,
            delay_frames: 0,
            invert_left_phase: false,
            invert_right_phase: false,
            mute: false,
            solo: false,
            bypass_effects: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundTrackSend {
    pub target: SoundTrackId,
    pub gain: f32,
    pub pre_effects: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundTrackMeter {
    pub track: SoundTrackId,
    pub peak_left: f32,
    pub peak_right: f32,
    pub rms_left: f32,
    pub rms_right: f32,
}

impl SoundTrackMeter {
    pub fn silent(track: SoundTrackId) -> Self {
        Self {
            track,
            peak_left: 0.0,
            peak_right: 0.0,
            rms_left: 0.0,
            rms_right: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundRayTracingConvolutionStatus {
    Disabled,
    WaitingForGeometryProvider,
    StaticImpulseResponse,
    RayTraced {
        cached_cells: usize,
        rays_per_update: usize,
    },
}
