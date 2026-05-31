use zircon_runtime::core::framework::sound::SoundTrackControls;

use super::super::dsp_state::SoundTrackRuntimeState;
use super::delay::delay_block;
use super::stereo::pan_stereo;

pub(crate) fn apply_track_controls(
    buffer: &mut [f32],
    channels: usize,
    controls: SoundTrackControls,
    state: &mut SoundTrackRuntimeState,
) {
    if controls.mute {
        buffer.fill(0.0);
        return;
    }
    if controls.delay_frames > 0 {
        delay_block(
            buffer,
            channels,
            controls.delay_frames,
            0.0,
            &mut state.control_delay_line,
        );
    }
    pan_stereo(
        buffer,
        channels,
        controls.pan,
        1.0,
        controls.left_gain * controls.gain,
        controls.right_gain * controls.gain,
        controls.invert_left_phase,
        controls.invert_right_phase,
    );
}
