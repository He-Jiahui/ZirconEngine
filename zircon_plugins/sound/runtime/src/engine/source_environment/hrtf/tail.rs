use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundHrtfProfileDescriptor, SoundListenerDescriptor, SoundSourceId,
};

use super::super::super::{SoundHrtfRenderState, SoundHrtfRenderStateKey};

pub(crate) fn hrtf_tail_pending_for_source(
    hrtf_states: &HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>,
    source_id: SoundSourceId,
    listener: Option<&SoundListenerDescriptor>,
    hrtf_profiles: &HashMap<String, SoundHrtfProfileDescriptor>,
) -> bool {
    let Some(listener) = listener else {
        return false;
    };
    let Some(profile_id) = listener.hrtf_profile.as_deref() else {
        return false;
    };
    if !hrtf_profiles.contains_key(profile_id) {
        return false;
    }
    let key = SoundHrtfRenderStateKey::new(source_id, listener.id, profile_id.to_string());
    hrtf_states
        .get(&key)
        .is_some_and(SoundHrtfRenderState::has_pending_tail)
}
