use zircon_runtime::core::framework::sound::{SoundError, SoundListenerDescriptor};

use crate::engine::SoundEngineState;

use super::common::validate_vec3;

pub(crate) fn validate_listener_descriptor(
    state: &SoundEngineState,
    listener: &SoundListenerDescriptor,
) -> Result<(), SoundError> {
    validate_vec3("listener position", listener.position)?;
    validate_vec3("listener forward", listener.forward)?;
    validate_vec3("listener up", listener.up)?;
    validate_vec3("listener left ear offset", listener.left_ear_offset)?;
    validate_vec3("listener right ear offset", listener.right_ear_offset)?;
    validate_vec3("listener velocity", listener.velocity)?;
    if !state
        .graph
        .tracks
        .iter()
        .any(|track| track.id == listener.mixer_target)
    {
        return Err(SoundError::UnknownTrack {
            track: listener.mixer_target,
        });
    }
    Ok(())
}
