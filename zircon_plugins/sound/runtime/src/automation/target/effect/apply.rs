use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectKind, SoundError, SoundParameterId,
};

use crate::engine::validation::validate_effect;

use super::{common, delay, dynamics, filter, gain, modulation, reverb, shaper, stereo};

pub(in crate::automation::target) fn apply_effect_parameter(
    effect: &mut SoundEffectDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    if common::apply_common_effect_parameter(effect, parameter, value)? {
        return Ok(());
    }

    match &mut effect.kind {
        SoundEffectKind::Gain(gain) => gain::apply_gain_parameter(gain, parameter, value)?,
        SoundEffectKind::Filter(filter) => {
            filter::apply_filter_parameter(filter, parameter, value)?
        }
        SoundEffectKind::Reverb(reverb) => {
            reverb::apply_reverb_parameter(reverb, parameter, value)?
        }
        SoundEffectKind::ConvolutionReverb(convolution) => {
            reverb::apply_convolution_reverb_parameter(convolution, parameter, value)?;
        }
        SoundEffectKind::Compressor(compressor) => {
            dynamics::apply_compressor_parameter(compressor, parameter, value)?;
        }
        SoundEffectKind::WaveShaper(shaper) => {
            shaper::apply_wave_shaper_parameter(shaper, parameter, value)?;
        }
        SoundEffectKind::Flanger(flanger) => {
            modulation::apply_flanger_parameter(flanger, parameter, value)?;
        }
        SoundEffectKind::Phaser(phaser) => {
            modulation::apply_phaser_parameter(phaser, parameter, value)?;
        }
        SoundEffectKind::Chorus(chorus) => {
            modulation::apply_chorus_parameter(chorus, parameter, value)?;
        }
        SoundEffectKind::Delay(delay) => delay::apply_delay_parameter(delay, parameter, value)?,
        SoundEffectKind::PanStereo(pan) => {
            stereo::apply_pan_stereo_parameter(pan, parameter, value)?
        }
        SoundEffectKind::Limiter(limiter) => {
            dynamics::apply_limiter_parameter(limiter, parameter, value)?;
        }
    }
    validate_effect(effect)
}
