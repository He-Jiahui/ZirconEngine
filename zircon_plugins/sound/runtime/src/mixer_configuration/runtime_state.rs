use zircon_runtime::core::framework::sound::SoundTrackMeter;

use crate::engine::SoundEngineState;

pub(crate) fn reset_mixer_runtime_state(
    state: &mut SoundEngineState,
    meters: Vec<SoundTrackMeter>,
) {
    state.effect_states.clear();
    state.track_states.clear();
    state.hrtf_states.clear();
    state.meters = meters;
    state.latency_frames = 0;
}
