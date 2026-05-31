use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundImpulseResponseId, SoundListenerDescriptor, SoundRayTracedImpulseResponseDescriptor,
    SoundSourceDescriptor, SoundSourceId, SoundVolumeId,
};

use super::super::super::math::{cross3, dot3, length3, normalize3, scale3, sub3};
use super::super::super::{occlusion_gain_for_query, SoundOcclusionQuery};
use super::{attenuation, cone, doppler};

#[derive(Clone, Copy, Debug)]
pub(in crate::engine::source_environment) struct SpatialProfile {
    pub(in crate::engine::source_environment) gain: f32,
    pub(in crate::engine::source_environment) pan: f32,
}

pub(in crate::engine::source_environment) fn spatial_profile(
    source_id: SoundSourceId,
    source: &SoundSourceDescriptor,
    listener: &SoundListenerDescriptor,
    spatial_scale: f32,
    volume: Option<SoundVolumeId>,
    ray_traced_impulse_responses: &HashMap<
        SoundImpulseResponseId,
        SoundRayTracedImpulseResponseDescriptor,
    >,
) -> SpatialProfile {
    let blend = source.spatial.spatial_blend.clamp(0.0, 1.0);
    if blend <= 0.0 {
        return SpatialProfile {
            gain: 1.0,
            pan: 0.0,
        };
    }

    let offset = scale3(sub3(source.position, listener.position), spatial_scale);
    let distance = length3(offset);
    let attenuation = attenuation::attenuation_gain(
        distance,
        source.spatial.min_distance,
        source.spatial.max_distance,
        source.spatial.attenuation,
    );
    let cone = cone::cone_gain(source.forward, source.position, listener.position, source);
    let occlusion = occlusion_gain_for_query(
        source.spatial.occlusion_enabled,
        SoundOcclusionQuery {
            source: source.id.unwrap_or(source_id),
            listener: Some(listener.id),
            volume,
        },
        ray_traced_impulse_responses,
    );
    let doppler = doppler::doppler_preview_gain(source, listener, offset);
    let listener_right = normalize3(cross3(listener.up, listener.forward));
    let direction = normalize3(offset);

    SpatialProfile {
        gain: ((1.0 - blend) + attenuation * blend) * cone * occlusion * doppler,
        pan: dot3(direction, listener_right).clamp(-1.0, 1.0) * blend,
    }
}
