use zircon_runtime::core::framework::sound::{
    SoundPlaybackFinishReason, SoundPlaybackFinished, SoundPlaybackId,
};

use super::super::super::state::SoundEngineState;

pub(super) fn finish_playbacks(
    state: &mut SoundEngineState,
    finished: Vec<(SoundPlaybackId, SoundPlaybackFinishReason)>,
) {
    for (playback_id, reason) in finished {
        if let Some(active) = state.playbacks.remove(&playback_id) {
            state.finished_playbacks.push(SoundPlaybackFinished {
                playback: playback_id,
                clip: active.clip,
                reason,
                completion_action: active.completion_action,
                output_track: active.output_track,
            });
        }
    }
}
