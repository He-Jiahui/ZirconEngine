use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundHrtfProfileDescriptor, SoundImpulseResponseId, SoundListenerDescriptor,
    SoundRayTracedImpulseResponseDescriptor, SoundSourceDescriptor, SoundSourceId,
    SoundVolumeDescriptor,
};

use crate::engine::{SoundHrtfRenderState, SoundHrtfRenderStateKey};

use super::super::{hrtf, spatial, volume};

#[derive(Clone, Copy, Debug)]
pub(super) struct ListenerEnvironmentProjection {
    pub(super) gain: f32,
    pub(super) pan: f32,
}

pub(super) fn apply_listener_environment(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    source_id: SoundSourceId,
    source: &SoundSourceDescriptor,
    listener: &SoundListenerDescriptor,
    spatial_scale: f32,
    volumes: &[SoundVolumeDescriptor],
    ray_traced_impulse_responses: &HashMap<
        SoundImpulseResponseId,
        SoundRayTracedImpulseResponseDescriptor,
    >,
    hrtf_profiles: &HashMap<String, SoundHrtfProfileDescriptor>,
    hrtf_states: &mut HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>,
) -> ListenerEnvironmentProjection {
    let spatial_scale = effective_spatial_scale(source, spatial_scale);
    let active_volume = volume::strongest_volume_influence(source.position, volumes);
    let spatial = spatial::spatial_profile(
        source_id,
        source,
        listener,
        spatial_scale,
        active_volume.as_ref().map(|volume| volume.descriptor.id),
        ray_traced_impulse_responses,
    );
    let hrtf_applied = if hrtf::apply_loaded_hrtf_profile_for_source(
        buffer,
        channels,
        source_id,
        listener,
        hrtf_profiles,
        hrtf_states,
    ) {
        true
    } else {
        hrtf::apply_hrtf_preview(
            buffer,
            channels,
            source,
            listener,
            sample_rate_hz,
            source.spatial.spatial_blend.clamp(0.0, 1.0),
            spatial_scale,
        )
    };

    ListenerEnvironmentProjection {
        gain: spatial.gain,
        pan: if hrtf_applied { 0.0 } else { spatial.pan },
    }
}

fn effective_spatial_scale(source: &SoundSourceDescriptor, fallback_spatial_scale: f32) -> f32 {
    source
        .spatial
        .spatial_scale
        .unwrap_or(fallback_spatial_scale)
        .max(0.0)
}
