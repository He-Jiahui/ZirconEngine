use std::collections::HashSet;

use crate::engine::SoundEngineState;

pub(crate) fn retain_timeline_sequences_for_automation_bindings(state: &mut SoundEngineState) {
    let automation_binding_ids = state
        .automation_bindings
        .keys()
        .copied()
        .collect::<HashSet<_>>();
    state.timeline_sequences.retain(|playback| {
        playback
            .sequence
            .tracks
            .iter()
            .all(|track| automation_binding_ids.contains(&track.binding))
    });
}
