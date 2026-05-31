use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor,
    SoundRayTracingConvolutionStatus,
};

pub(crate) fn refresh_ray_tracing_status(
    status: &mut SoundRayTracingConvolutionStatus,
    descriptors: &HashMap<SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor>,
) {
    if descriptors.is_empty() {
        if matches!(status, SoundRayTracingConvolutionStatus::RayTraced { .. }) {
            *status = SoundRayTracingConvolutionStatus::WaitingForGeometryProvider;
        }
        return;
    }

    let rays_per_update = descriptors
        .values()
        .map(|descriptor| descriptor.rays_traced)
        .max()
        .unwrap_or(1);
    *status = SoundRayTracingConvolutionStatus::RayTraced {
        cached_cells: descriptors.len(),
        rays_per_update,
    };
}
