mod apply;
mod profile;

use zircon_runtime::core::framework::sound::{SoundListenerDescriptor, SoundSourceDescriptor};

use crate::engine::hrtf::clear_non_binaural_output_channels;

use apply::apply_hrtf_preview_profile;
use profile::hrtf_preview_profile;

pub(in crate::engine::source_environment) fn apply_hrtf_preview(
    buffer: &mut [f32],
    channels: usize,
    source: &SoundSourceDescriptor,
    listener: &SoundListenerDescriptor,
    sample_rate_hz: u32,
    blend: f32,
    spatial_scale: f32,
) -> bool {
    if channels < 2 {
        return false;
    }
    if listener
        .hrtf_profile
        .as_deref()
        .unwrap_or_default()
        .is_empty()
    {
        return false;
    }
    let profile = hrtf_preview_profile(source, listener, sample_rate_hz, blend, spatial_scale);
    if channels > 2 {
        clear_non_binaural_output_channels(buffer, channels);
        return true;
    }
    if profile.is_identity() {
        return true;
    }

    apply_hrtf_preview_profile(buffer, channels, profile);
    true
}
