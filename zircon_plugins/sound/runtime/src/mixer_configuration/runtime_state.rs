use zircon_runtime::core::framework::sound::SoundTrackMeter;

use crate::engine::SoundEngineState;

pub(crate) fn reset_mixer_runtime_state(
    state: &mut SoundEngineState,
    meters: Vec<SoundTrackMeter>,
) {
    state.hrtf_states.clear();
    state.meters = meters;
    state.latency_frames = 0;
}
