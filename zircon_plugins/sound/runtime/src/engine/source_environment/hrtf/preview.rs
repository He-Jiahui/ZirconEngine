mod apply;
mod profile;

use zircon_runtime::core::framework::sound::{SoundListenerDescriptor, SoundSourceDescriptor};

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
) {
    if channels < 2 {
        return;
    }

    let profile = hrtf_preview_profile(source, listener, sample_rate_hz, blend, spatial_scale);
    if profile.is_identity() && channels <= 2 {
        return;
    }

    apply_hrtf_preview_profile(buffer, channels, profile);
}
