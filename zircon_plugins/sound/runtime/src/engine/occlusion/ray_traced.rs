use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor,
};

use super::query::SoundOcclusionQuery;

pub(super) fn ray_traced_occlusion_gain(
    query: SoundOcclusionQuery,
    ray_traced_impulse_responses: &HashMap<
        SoundImpulseResponseId,
        SoundRayTracedImpulseResponseDescriptor,
    >,
) -> Option<f32> {
    ray_traced_impulse_responses
        .values()
        .filter_map(|descriptor| match descriptor.occlusion_gain {
            Some(gain) if descriptor_matches_query(descriptor, query) => Some((
                descriptor_specificity(descriptor),
                descriptor.rays_traced,
                gain,
            )),
            _ => None,
        })
        .max_by_key(|(specificity, rays_traced, _)| (*specificity, *rays_traced))
        .map(|(_, _, gain)| gain)
}

fn descriptor_matches_query(
    descriptor: &SoundRayTracedImpulseResponseDescriptor,
    query: SoundOcclusionQuery,
) -> bool {
    descriptor
        .source
        .map_or(true, |source| source == query.source)
        && descriptor
            .listener
            .map_or(true, |listener| Some(listener) == query.listener)
        && descriptor
            .volume
            .map_or(true, |volume| Some(volume) == query.volume)
}

fn descriptor_specificity(descriptor: &SoundRayTracedImpulseResponseDescriptor) -> u8 {
    descriptor.source.is_some() as u8
        + descriptor.listener.is_some() as u8
        + descriptor.volume.is_some() as u8
}
