use crate::graphics::resource_limits::HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;
use crate::graphics::types::GraphicsError;

const REQUIRED_RENDER_BIND_GROUP_LIMIT: u32 = 5;
const REQUIRED_POST_PROCESS_SAMPLED_TEXTURES_PER_SHADER_STAGE: u32 = 20;

pub(super) fn request_device(
    adapter: &wgpu::Adapter,
) -> Result<(wgpu::Device, wgpu::Queue), GraphicsError> {
    let mut requested_features = wgpu::Features::empty();
    let adapter_features = adapter.features();
    if adapter_features.contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT) {
        requested_features |= wgpu::Features::MULTI_DRAW_INDIRECT_COUNT;
    }
    if adapter_features.contains(wgpu::Features::INDIRECT_FIRST_INSTANCE) {
        requested_features |= wgpu::Features::INDIRECT_FIRST_INSTANCE;
    }
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-device"),
        required_features: requested_features,
        required_limits: required_render_limits(&adapter.limits()),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .map_err(GraphicsError::from)
}

fn required_render_limits(adapter_limits: &wgpu::Limits) -> wgpu::Limits {
    let mut limits = wgpu::Limits {
        max_bind_groups: REQUIRED_RENDER_BIND_GROUP_LIMIT,
        max_sampled_textures_per_shader_stage:
            REQUIRED_POST_PROCESS_SAMPLED_TEXTURES_PER_SHADER_STAGE,
        ..wgpu::Limits::default()
    };
    if adapter_limits.max_storage_buffers_per_shader_stage
        >= HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
    {
        limits.max_storage_buffers_per_shader_stage =
            HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;
    }
    limits
}

#[cfg(test)]
mod tests {
    use crate::graphics::resource_limits::HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;

    use super::{
        required_render_limits, REQUIRED_POST_PROCESS_SAMPLED_TEXTURES_PER_SHADER_STAGE,
        REQUIRED_RENDER_BIND_GROUP_LIMIT,
    };

    #[test]
    fn offscreen_device_limits_cover_renderer_layout_requirements() {
        let limits = required_render_limits(&wgpu::Limits {
            max_storage_buffers_per_shader_stage:
                HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            ..wgpu::Limits::default()
        });

        assert!(limits.max_bind_groups >= REQUIRED_RENDER_BIND_GROUP_LIMIT);
        assert!(
            limits.max_sampled_textures_per_shader_stage
                >= REQUIRED_POST_PROCESS_SAMPLED_TEXTURES_PER_SHADER_STAGE
        );
        assert!(
            limits.max_storage_buffers_per_shader_stage
                >= HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
    }

    #[test]
    fn offscreen_device_limits_keep_hzb_optional_when_adapter_limit_is_lower() {
        let limits = required_render_limits(&wgpu::Limits::default());

        assert!(
            limits.max_storage_buffers_per_shader_stage
                < HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
    }
}
