use zircon_runtime::core::framework::sound::{SoundEffectDescriptor, SoundError, SoundParameterId};

use crate::engine::validation::validate_effect;

use super::super::helpers::bool_from_value;

pub(super) fn apply_common_effect_parameter(
    effect: &mut SoundEffectDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<bool, SoundError> {
    match parameter.as_str() {
        "enabled" => {
            effect.enabled = bool_from_value(value);
            Ok(true)
        }
        "bypass" => {
            effect.bypass = bool_from_value(value);
            Ok(true)
        }
        "wet" => {
            effect.wet = value;
            validate_effect(effect)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
