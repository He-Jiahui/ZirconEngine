use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundHrtfProfileDescriptor, SoundListenerDescriptor, SoundSourceId,
};

use super::super::super::hrtf::apply_loaded_hrtf_profile;
use super::super::super::{SoundHrtfRenderState, SoundHrtfRenderStateKey};

pub(in crate::engine::source_environment) fn apply_loaded_hrtf_profile_for_source(
    buffer: &mut [f32],
    channels: usize,
    source_id: SoundSourceId,
    listener: &SoundListenerDescriptor,
    hrtf_profiles: &HashMap<String, SoundHrtfProfileDescriptor>,
    hrtf_states: &mut HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>,
) -> bool {
    let Some(profile_id) = listener
        .hrtf_profile
        .as_deref()
        .filter(|profile_id| !profile_id.is_empty())
    else {
        return false;
    };
    let Some(profile) = hrtf_profiles.get(profile_id) else {
        return false;
    };

    let key = SoundHrtfRenderStateKey::new(source_id, listener.id, profile_id.to_string());
    let state = hrtf_states.entry(key).or_default();
    apply_loaded_hrtf_profile(buffer, channels, profile, state)
}
