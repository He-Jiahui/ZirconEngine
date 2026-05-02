use serde::{Deserialize, Serialize};

use super::{
    ExternalAudioSourceHandle, SoundClipId, SoundImpulseResponseId, SoundListenerId,
    SoundParameterId, SoundSourceId, SoundTrackId, SoundVolumeId,
};

pub const AUDIO_SOURCE_COMPONENT_TYPE: &str = "sound.Component.AudioSource";
pub const AUDIO_LISTENER_COMPONENT_TYPE: &str = "sound.Component.AudioListener";
pub const AUDIO_VOLUME_COMPONENT_TYPE: &str = "sound.Component.AudioVolume";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundSourceDescriptor {
    pub id: Option<SoundSourceId>,
    pub input: SoundSourceInput,
    pub output_track: SoundTrackId,
    pub sends: Vec<SoundSourceSend>,
    pub position: [f32; 3],
    pub forward: [f32; 3],
    pub velocity: [f32; 3],
    pub gain: f32,
    pub looped: bool,
    pub playing: bool,
    pub spatial: SoundSpatialSourceSettings,
    pub parameter_bindings: Vec<SoundSourceParameterBinding>,
}

impl SoundSourceDescriptor {
    pub fn clip(clip: SoundClipId) -> Self {
        Self {
            id: None,
            input: SoundSourceInput::Clip(clip),
            output_track: SoundTrackId::master(),
            sends: Vec::new(),
            position: [0.0, 0.0, 0.0],
            forward: [0.0, 0.0, 1.0],
            velocity: [0.0, 0.0, 0.0],
            gain: 1.0,
            looped: false,
            playing: true,
            spatial: SoundSpatialSourceSettings::default(),
            parameter_bindings: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SoundSourceInput {
    Clip(SoundClipId),
    External(ExternalAudioSourceHandle),
    SynthParameter {
        parameter: SoundParameterId,
        default_value: f32,
    },
    Silence,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundSourceSend {
    pub target: SoundTrackId,
    pub gain: f32,
    pub pre_spatial: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundSourceParameterBinding {
    pub source_parameter: SoundParameterId,
    pub synth_parameter: SoundParameterId,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundSpatialSourceSettings {
    pub spatial_blend: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub attenuation: SoundAttenuationMode,
    pub cone_inner_degrees: f32,
    pub cone_outer_degrees: f32,
    pub doppler_factor: f32,
    pub occlusion_enabled: bool,
    pub convolution_send: Option<SoundImpulseResponseId>,
}

impl Default for SoundSpatialSourceSettings {
    fn default() -> Self {
        Self {
            spatial_blend: 0.0,
            min_distance: 1.0,
            max_distance: 50.0,
            attenuation: SoundAttenuationMode::InverseDistance,
            cone_inner_degrees: 360.0,
            cone_outer_degrees: 360.0,
            doppler_factor: 1.0,
            occlusion_enabled: false,
            convolution_send: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundAttenuationMode {
    None,
    Linear,
    InverseDistance,
    InverseDistanceSquared,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundListenerDescriptor {
    pub id: SoundListenerId,
    pub active: bool,
    pub position: [f32; 3],
    pub forward: [f32; 3],
    pub up: [f32; 3],
    pub left_ear_offset: [f32; 3],
    pub right_ear_offset: [f32; 3],
    pub velocity: [f32; 3],
    pub hrtf_profile: Option<String>,
    pub doppler_tracking: bool,
    pub mixer_target: SoundTrackId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundVolumeDescriptor {
    pub id: SoundVolumeId,
    pub shape: SoundVolumeShape,
    pub priority: i32,
    pub interior_gain: f32,
    pub exterior_gain: f32,
    pub low_pass_cutoff_hz: Option<f32>,
    pub reverb_send: f32,
    pub convolution_send: Option<SoundImpulseResponseId>,
    pub crossfade_distance: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SoundVolumeShape {
    Sphere { center: [f32; 3], radius: f32 },
    Box { center: [f32; 3], extents: [f32; 3] },
}
