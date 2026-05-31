use zircon_runtime::core::framework::sound::{SoundError, SoundTrackDescriptor, SoundTrackSend};

use super::values::validate_pan_value;

pub(super) fn validate_track_controls(track: &SoundTrackDescriptor) -> Result<(), SoundError> {
    let controls = track.controls;
    let gain = controls.gain;
    let pan = controls.pan;
    let left_gain = controls.left_gain;
    let right_gain = controls.right_gain;
    if !gain.is_finite() || !left_gain.is_finite() || !right_gain.is_finite() {
        return Err(SoundError::InvalidMixerGraph(format!(
            "track {} controls gain and L/R trims must be finite",
            track.display_name
        )));
    }
    validate_pan_value("track pan", pan).map_err(SoundError::InvalidMixerGraph)
}

pub(super) fn validate_track_send(
    track: &SoundTrackDescriptor,
    send: &SoundTrackSend,
) -> Result<(), SoundError> {
    if send.gain.is_finite() {
        return Ok(());
    }
    Err(SoundError::InvalidMixerGraph(format!(
        "track {} send gain to {:?} must be finite",
        track.display_name, send.target
    )))
}
