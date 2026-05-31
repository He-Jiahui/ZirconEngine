use zircon_runtime::core::framework::sound::{SoundError, SoundSourceInput};

use crate::engine::SoundEngineState;

use super::super::external_source::validate_external_source_handle;

pub(super) fn validate_source_input(
    state: &SoundEngineState,
    input: &SoundSourceInput,
) -> Result<(), SoundError> {
    match input {
        SoundSourceInput::Clip(clip) => {
            if !state.clips.contains_key(clip) {
                return Err(SoundError::UnknownClip { clip: *clip });
            }
        }
        SoundSourceInput::External(handle) => validate_external_source_handle(handle)?,
        SoundSourceInput::SynthParameter {
            parameter,
            default_value,
        } => {
            if parameter.as_str().trim().is_empty() || !default_value.is_finite() {
                return Err(SoundError::InvalidParameter(
                    "synth source input requires a non-empty parameter id and finite default value"
                        .to_string(),
                ));
            }
        }
        SoundSourceInput::Silence => {}
    }
    Ok(())
}
