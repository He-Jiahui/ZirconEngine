use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor,
};

use super::constants::OCCLUSION_FALLBACK_GAIN;
use super::query::SoundOcclusionQuery;
use super::ray_traced::ray_traced_occlusion_gain;

pub(crate) fn occlusion_gain_for_query(
    occlusion_enabled: bool,
    query: SoundOcclusionQuery,
    ray_traced_impulse_responses: &HashMap<
        SoundImpulseResponseId,
        SoundRayTracedImpulseResponseDescriptor,
    >,
) -> f32 {
    if !occlusion_enabled {
        return 1.0;
    }

    ray_traced_occlusion_gain(query, ray_traced_impulse_responses)
        .unwrap_or(OCCLUSION_FALLBACK_GAIN)
}
