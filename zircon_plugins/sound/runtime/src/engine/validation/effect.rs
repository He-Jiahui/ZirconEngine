use zircon_runtime::core::framework::sound::{SoundEffectDescriptor, SoundEffectKind, SoundError};

use super::values::{
    validate_feedback_effect_value, validate_finite_effect_value,
    validate_non_negative_effect_value, validate_pan_value, validate_unit_effect_value,
};

pub(crate) fn validate_effect(effect: &SoundEffectDescriptor) -> Result<(), SoundError> {
    if !effect.wet.is_finite() || !(0.0..=1.0).contains(&effect.wet) {
        return Err(SoundError::InvalidEffect(format!(
            "effect {} wet mix must be finite and between 0 and 1",
            effect.display_name
        )));
    }
    match &effect.kind {
        SoundEffectKind::Gain(gain) => validate_finite_effect_value(effect, "gain", gain.gain),
        SoundEffectKind::Filter(filter) => {
            if !filter.cutoff_hz.is_finite()
                || filter.cutoff_hz <= 0.0
                || !filter.resonance.is_finite()
                || filter.resonance < 0.0
                || !filter.gain_db.is_finite()
            {
                return Err(SoundError::InvalidEffect(
                    "filter cutoff, resonance, and gain must be finite, with positive cutoff and non-negative resonance"
                        .to_string(),
                ));
            }
            Ok(())
        }
        SoundEffectKind::Reverb(reverb) => {
            validate_unit_effect_value(effect, "room size", reverb.room_size)?;
            validate_unit_effect_value(effect, "damping", reverb.damping)
        }
        SoundEffectKind::ConvolutionReverb(_) => Ok(()),
        SoundEffectKind::Compressor(compressor) => {
            validate_finite_effect_value(effect, "threshold dB", compressor.threshold_db)?;
            validate_finite_effect_value(effect, "ratio", compressor.ratio)?;
            if compressor.ratio < 1.0 {
                return Err(SoundError::InvalidEffect(
                    "compressor ratio must be at least 1".to_string(),
                ));
            }
            validate_non_negative_effect_value(effect, "attack ms", compressor.attack_ms)?;
            validate_non_negative_effect_value(effect, "release ms", compressor.release_ms)?;
            validate_finite_effect_value(effect, "makeup gain dB", compressor.makeup_gain_db)
        }
        SoundEffectKind::WaveShaper(shaper) => {
            validate_non_negative_effect_value(effect, "drive", shaper.drive)
        }
        SoundEffectKind::Flanger(flanger) => {
            validate_non_negative_effect_value(effect, "rate Hz", flanger.rate_hz)?;
            validate_feedback_effect_value(effect, "feedback", flanger.feedback)
        }
        SoundEffectKind::Phaser(phaser) => {
            validate_non_negative_effect_value(effect, "rate Hz", phaser.rate_hz)?;
            validate_unit_effect_value(effect, "depth", phaser.depth)?;
            validate_feedback_effect_value(effect, "feedback", phaser.feedback)?;
            validate_finite_effect_value(effect, "phase offset", phaser.phase_offset)
        }
        SoundEffectKind::Chorus(chorus) => {
            if chorus.voices == 0 {
                return Err(SoundError::InvalidEffect(
                    "chorus must have at least one voice".to_string(),
                ));
            }
            validate_non_negative_effect_value(effect, "rate Hz", chorus.rate_hz)
        }
        SoundEffectKind::Delay(delay) => {
            validate_feedback_effect_value(effect, "feedback", delay.feedback)
        }
        SoundEffectKind::PanStereo(pan) => {
            validate_pan_value("stereo pan", pan.pan).map_err(SoundError::InvalidEffect)?;
            validate_non_negative_effect_value(effect, "stereo width", pan.width)?;
            validate_finite_effect_value(effect, "left gain", pan.left_gain)?;
            validate_finite_effect_value(effect, "right gain", pan.right_gain)
        }
        SoundEffectKind::Limiter(limiter) => {
            if !limiter.ceiling.is_finite() || limiter.ceiling <= 0.0 {
                return Err(SoundError::InvalidEffect(
                    "limiter ceiling must be finite and positive".to_string(),
                ));
            }
            Ok(())
        }
    }
}
