use crate::core::framework::render::{RenderDeviceDiagnostics, RenderDeviceLimitDiagnostics};
use crate::rhi::RenderBackendCaps;

/// Projects actual RHI device facts into the framework-facing render snapshot.
pub(in crate::graphics::runtime::render_framework) fn render_device_diagnostics(
    caps: &RenderBackendCaps,
) -> Option<RenderDeviceDiagnostics> {
    let adapter = caps.adapter.as_ref()?;
    let limits = caps.device_limits.as_ref()?;
    if adapter.name.trim().is_empty() || adapter.device_type.trim().is_empty() {
        return None;
    }

    Some(RenderDeviceDiagnostics {
        adapter_name: adapter.name.clone(),
        adapter_device_type: adapter.device_type.clone(),
        limits: RenderDeviceLimitDiagnostics {
            max_bind_groups: limits.max_bind_groups,
            max_texture_dimension_2d: limits.max_texture_dimension_2d,
            max_texture_array_layers: limits.max_texture_array_layers,
            max_sampled_textures_per_shader_stage: limits.max_sampled_textures_per_shader_stage,
            max_binding_array_elements_per_shader_stage: limits
                .max_binding_array_elements_per_shader_stage,
            max_binding_array_sampler_elements_per_shader_stage: limits
                .max_binding_array_sampler_elements_per_shader_stage,
            max_storage_buffers_per_shader_stage: limits.max_storage_buffers_per_shader_stage,
            max_storage_buffer_binding_size: limits.max_storage_buffer_binding_size,
        },
    })
}

#[cfg(test)]
#[path = "tests/device_diagnostics.rs"]
mod tests;
