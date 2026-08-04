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
mod tests {
    use crate::rhi::{RenderAdapterInfo, RenderBackendCaps, RenderDeviceLimits};

    use super::render_device_diagnostics;

    #[test]
    fn render_device_diagnostics_projects_actual_rhi_device_facts() {
        let caps = RenderBackendCaps::new("wgpu(dx12)")
            .with_adapter(RenderAdapterInfo {
                name: "Zircon Test Adapter".to_owned(),
                device_type: "discrete_gpu".to_owned(),
            })
            .with_device_limits(RenderDeviceLimits {
                max_bind_groups: 5,
                max_texture_dimension_2d: 16_384,
                max_texture_array_layers: 256,
                max_sampled_textures_per_shader_stage: 16,
                max_binding_array_elements_per_shader_stage: 500_000,
                max_binding_array_sampler_elements_per_shader_stage: 1_000,
                max_storage_buffers_per_shader_stage: 8,
                max_storage_buffer_binding_size: 134_217_728,
            });

        let diagnostics = render_device_diagnostics(&caps).expect("device diagnostics");

        assert_eq!(diagnostics.adapter_name, "Zircon Test Adapter");
        assert_eq!(diagnostics.adapter_device_type, "discrete_gpu");
        assert_eq!(diagnostics.limits.max_bind_groups, 5);
        assert_eq!(diagnostics.limits.max_texture_dimension_2d, 16_384);
        assert_eq!(diagnostics.limits.max_texture_array_layers, 256);
        assert_eq!(diagnostics.limits.max_sampled_textures_per_shader_stage, 16);
        assert_eq!(
            diagnostics
                .limits
                .max_binding_array_elements_per_shader_stage,
            500_000
        );
        assert_eq!(
            diagnostics
                .limits
                .max_binding_array_sampler_elements_per_shader_stage,
            1_000
        );
        assert_eq!(diagnostics.limits.max_storage_buffers_per_shader_stage, 8);
        assert_eq!(
            diagnostics.limits.max_storage_buffer_binding_size,
            134_217_728
        );
    }

    #[test]
    fn render_device_diagnostics_refuses_incomplete_rhi_device_facts() {
        assert!(render_device_diagnostics(&RenderBackendCaps::new("test")).is_none());
    }
}
