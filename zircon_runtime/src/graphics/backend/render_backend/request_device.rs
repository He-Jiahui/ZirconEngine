use crate::graphics::types::GraphicsError;

const REQUIRED_RENDER_BIND_GROUP_LIMIT: u32 = 5;
const REQUIRED_POST_PROCESS_SAMPLED_TEXTURES_PER_SHADER_STAGE: u32 = 20;

pub(super) fn request_device(
    adapter: &wgpu::Adapter,
) -> Result<(wgpu::Device, wgpu::Queue), GraphicsError> {
    let mut requested_features = wgpu::Features::empty();
    let adapter_features = adapter.features();
    if adapter_features.contains(wgpu::Features::INDIRECT_FIRST_INSTANCE) {
        requested_features |= wgpu::Features::INDIRECT_FIRST_INSTANCE;
    }
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-device"),
        required_features: requested_features,
        required_limits: required_render_limits(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .map_err(GraphicsError::from)
}

fn required_render_limits() -> wgpu::Limits {
    wgpu::Limits {
        max_bind_groups: REQUIRED_RENDER_BIND_GROUP_LIMIT,
        max_sampled_textures_per_shader_stage:
            REQUIRED_POST_PROCESS_SAMPLED_TEXTURES_PER_SHADER_STAGE,
        ..wgpu::Limits::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        required_render_limits, REQUIRED_POST_PROCESS_SAMPLED_TEXTURES_PER_SHADER_STAGE,
        REQUIRED_RENDER_BIND_GROUP_LIMIT,
    };

    #[test]
    fn offscreen_device_limits_cover_renderer_layout_requirements() {
        let limits = required_render_limits();

        assert!(limits.max_bind_groups >= REQUIRED_RENDER_BIND_GROUP_LIMIT);
        assert!(
            limits.max_sampled_textures_per_shader_stage
                >= REQUIRED_POST_PROCESS_SAMPLED_TEXTURES_PER_SHADER_STAGE
        );
    }
}
