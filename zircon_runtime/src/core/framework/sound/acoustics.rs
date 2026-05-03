use serde::{Deserialize, Serialize};

use super::{SoundImpulseResponseId, SoundListenerId, SoundSourceId, SoundVolumeId};

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundRayTracedImpulseResponseDescriptor {
    pub impulse_response: SoundImpulseResponseId,
    pub cell_key: String,
    pub source: Option<SoundSourceId>,
    pub listener: Option<SoundListenerId>,
    pub volume: Option<SoundVolumeId>,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub rays_traced: usize,
    pub samples: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundHrtfProfileDescriptor {
    pub profile_id: String,
    pub display_name: String,
    pub sample_rate_hz: u32,
    pub left_kernel: Vec<f32>,
    pub right_kernel: Vec<f32>,
    pub notes: Vec<String>,
}
