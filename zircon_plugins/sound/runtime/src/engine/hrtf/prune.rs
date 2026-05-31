use std::collections::{HashMap, HashSet};

use super::{SoundHrtfRenderState, SoundHrtfRenderStateKey};

pub(crate) fn prune_hrtf_render_states(
    states: &mut HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>,
    active_keys: &HashSet<SoundHrtfRenderStateKey>,
) {
    states.retain(|key, _| active_keys.contains(key));
}
