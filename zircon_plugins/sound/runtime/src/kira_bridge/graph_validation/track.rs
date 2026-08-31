//! Validates track controls and sends before Kira allocation.

use zircon_runtime::core::framework::sound::{
    SoundError, SoundTrackControls, SoundTrackDescriptor, SoundTrackSend,
};

use super::values::{validate_graph_history_frames, validate_pan_value};

pub(crate) fn validate_track_controls(
    display_name: &str,
    controls: SoundTrackControls,
) -> Result<(), SoundError> {
    let gain = controls.gain;
    let pan = controls.pan;
    let left_gain = controls.left_gain;
    let right_gain = controls.right_gain;
    if !gain.is_finite() || !left_gain.is_finite() || !right_gain.is_finite() {
        return Err(SoundError::InvalidMixerGraph(format!(
            "track {} controls gain and L/R trims must be finite",
            display_name
        )));
    }
    validate_graph_history_frames(
        &format!("track {display_name} delay frames"),
        controls.delay_frames,
    )?;
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
