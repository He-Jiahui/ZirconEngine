use serde::{Deserialize, Serialize};

use super::{SoundEffectId, SoundImpulseResponseId, SoundTrackId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundEffectDescriptor {
    pub id: SoundEffectId,
    pub display_name: String,
    pub enabled: bool,
    pub bypass: bool,
    pub wet: f32,
    pub kind: SoundEffectKind,
}

impl SoundEffectDescriptor {
    pub fn new(id: SoundEffectId, display_name: impl Into<String>, kind: SoundEffectKind) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            enabled: true,
            bypass: false,
            wet: 1.0,
            kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SoundEffectKind {
    Gain(SoundGainEffect),
    Filter(SoundFilterEffect),
    Reverb(SoundReverbEffect),
    ConvolutionReverb(SoundConvolutionReverbEffect),
    Compressor(SoundCompressorEffect),
    WaveShaper(SoundWaveShaperEffect),
    Flanger(SoundFlangerEffect),
    Phaser(SoundPhaserEffect),
    Chorus(SoundChorusEffect),
    Delay(SoundDelayEffect),
    PanStereo(SoundPanStereoEffect),
    Limiter(SoundLimiterEffect),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundGainEffect {
    pub gain: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundFilterMode {
    LowPass,
    HighPass,
    BandPass,
    Notch,
    LowShelf,
    HighShelf,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundFilterEffect {
    pub mode: SoundFilterMode,
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub gain_db: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundReverbEffect {
    pub room_size: f32,
    pub damping: f32,
    pub pre_delay_frames: usize,
    pub tail_frames: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundConvolutionReverbEffect {
    pub impulse_response: SoundImpulseResponseId,
    pub fallback_to_algorithmic: bool,
    pub latency_frames: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundCompressorEffect {
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_gain_db: f32,
    pub sidechain: Option<SoundSidechainInput>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundSidechainInput {
    pub track: SoundTrackId,
    pub pre_effects: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundWaveShaperEffect {
    pub drive: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundFlangerEffect {
    pub delay_frames: usize,
    pub depth_frames: usize,
    pub rate_hz: f32,
    pub feedback: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundPhaserEffect {
    pub rate_hz: f32,
    pub depth: f32,
    pub feedback: f32,
    pub phase_offset: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundChorusEffect {
    pub voices: u8,
    pub delay_frames: usize,
    pub depth_frames: usize,
    pub rate_hz: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundDelayEffect {
    pub delay_frames: usize,
    pub feedback: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundPanStereoEffect {
    pub pan: f32,
    pub width: f32,
    pub left_gain: f32,
    pub right_gain: f32,
    pub invert_left_phase: bool,
    pub invert_right_phase: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundLimiterEffect {
    pub ceiling: f32,
}
