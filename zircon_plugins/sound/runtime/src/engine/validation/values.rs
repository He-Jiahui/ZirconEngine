use zircon_runtime::core::framework::sound::{SoundEffectDescriptor, SoundError};

const DEFAULT_HISTORY_BUDGET_SAMPLE_RATE_HZ: usize = 48_000;
const MAX_HISTORY_BUDGET_SECONDS: usize = 3;
const MAX_RUNTIME_HISTORY_FRAMES: usize =
    DEFAULT_HISTORY_BUDGET_SAMPLE_RATE_HZ * MAX_HISTORY_BUDGET_SECONDS;

pub(super) fn validate_finite_effect_value(
    effect: &SoundEffectDescriptor,
    label: &str,
    value: f32,
) -> Result<(), SoundError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SoundError::InvalidEffect(format!(
            "effect {} {label} must be finite",
            effect.display_name
        )))
    }
}

pub(super) fn validate_non_negative_effect_value(
    effect: &SoundEffectDescriptor,
    label: &str,
    value: f32,
) -> Result<(), SoundError> {
    validate_finite_effect_value(effect, label, value)?;
    if value >= 0.0 {
        Ok(())
    } else {
        Err(SoundError::InvalidEffect(format!(
            "effect {} {label} must be non-negative",
            effect.display_name
        )))
    }
}

pub(super) fn validate_unit_effect_value(
    effect: &SoundEffectDescriptor,
    label: &str,
    value: f32,
) -> Result<(), SoundError> {
    validate_finite_effect_value(effect, label, value)?;
    if (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(SoundError::InvalidEffect(format!(
            "effect {} {label} must be between 0 and 1",
            effect.display_name
        )))
    }
}

pub(super) fn validate_feedback_effect_value(
    effect: &SoundEffectDescriptor,
    label: &str,
    value: f32,
) -> Result<(), SoundError> {
    validate_finite_effect_value(effect, label, value)?;
    if (0.0..=0.99).contains(&value) {
        Ok(())
    } else {
        Err(SoundError::InvalidEffect(format!(
            "effect {} {label} must be between 0 and 0.99",
            effect.display_name
        )))
    }
}

pub(super) fn validate_effect_history_frames(
    effect: &SoundEffectDescriptor,
    label: &str,
    frames: usize,
) -> Result<(), SoundError> {
    validate_effect_history_frame_total(effect, label, Some(frames))
}

pub(super) fn validate_effect_history_frame_total(
    effect: &SoundEffectDescriptor,
    label: &str,
    frames: Option<usize>,
) -> Result<(), SoundError> {
    if frames
        .map(|frames| frames <= MAX_RUNTIME_HISTORY_FRAMES)
        .unwrap_or(false)
    {
        Ok(())
    } else {
        Err(SoundError::InvalidEffect(format!(
            "effect {} {label} must fit the runtime history budget of {MAX_RUNTIME_HISTORY_FRAMES} frames",
            effect.display_name
        )))
    }
}

pub(super) fn validate_graph_history_frames(label: &str, frames: usize) -> Result<(), SoundError> {
    if frames <= MAX_RUNTIME_HISTORY_FRAMES {
        Ok(())
    } else {
        Err(SoundError::InvalidMixerGraph(format!(
            "{label} must fit the runtime history budget of {MAX_RUNTIME_HISTORY_FRAMES} frames"
        )))
    }
}

pub(super) fn validate_pan_value(label: &str, value: f32) -> Result<(), String> {
    if value.is_finite() && (-1.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(format!("{label} must be finite and between -1 and 1"))
    }
}
