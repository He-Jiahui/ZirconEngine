use crate::graphics::resource_limits::{
    HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
    MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
    POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE,
    REFLECTION_PROBE_REQUIRED_TEXTURE_ARRAY_LAYERS,
};
use crate::graphics::types::GraphicsError;

const REQUIRED_RENDER_BIND_GROUP_LIMIT: u32 = 5;

pub(super) fn request_device(
    adapter: &wgpu::Adapter,
) -> Result<(wgpu::Device, wgpu::Queue), GraphicsError> {
    let adapter_features = adapter.features();
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-device"),
        required_features: required_render_features(adapter_features),
        required_limits: required_render_limits(&adapter.limits()),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .map_err(GraphicsError::from)
}

fn required_render_features(adapter_features: wgpu::Features) -> wgpu::Features {
    let mut requested_features = wgpu::Features::RG11B10UFLOAT_RENDERABLE;
    for feature in [
        wgpu::Features::MULTI_DRAW_INDIRECT_COUNT,
        wgpu::Features::INDIRECT_FIRST_INSTANCE,
    ] {
        if adapter_features.contains(feature) {
            requested_features |= feature;
        }
    }
    requested_features
}

fn required_render_limits(adapter_limits: &wgpu::Limits) -> wgpu::Limits {
    let mut limits = wgpu::Limits {
        max_bind_groups: REQUIRED_RENDER_BIND_GROUP_LIMIT,
        max_sampled_textures_per_shader_stage:
            POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE,
        max_texture_array_layers: REFLECTION_PROBE_REQUIRED_TEXTURE_ARRAY_LAYERS,
        ..wgpu::Limits::default()
    };
    let required_storage_buffers_per_shader_stage =
        HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
            .max(MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE);
    if adapter_limits.max_storage_buffers_per_shader_stage
        >= required_storage_buffers_per_shader_stage
    {
        limits.max_storage_buffers_per_shader_stage = required_storage_buffers_per_shader_stage;
    } else if adapter_limits.max_storage_buffers_per_shader_stage
        >= MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
    {
        limits.max_storage_buffers_per_shader_stage =
            MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;
    }
    limits
}

#[cfg(test)]
mod tests {
    use crate::graphics::resource_limits::{
        HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
        MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
        POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE,
        REFLECTION_PROBE_REQUIRED_TEXTURE_ARRAY_LAYERS,
    };

    use super::{
        required_render_features, required_render_limits, REQUIRED_RENDER_BIND_GROUP_LIMIT,
    };

    #[test]
    fn offscreen_device_features_request_rg11b10_render_target_when_available() {
        let features = required_render_features(
            wgpu::Features::RG11B10UFLOAT_RENDERABLE | wgpu::Features::INDIRECT_FIRST_INSTANCE,
        );

        assert!(features.contains(wgpu::Features::RG11B10UFLOAT_RENDERABLE));
        assert!(features.contains(wgpu::Features::INDIRECT_FIRST_INSTANCE));
        assert!(!features.contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT));
    }

    #[test]
    fn offscreen_device_features_require_rg11b10_render_target_for_post_process() {
        let features = required_render_features(wgpu::Features::empty());

        assert!(features.contains(wgpu::Features::RG11B10UFLOAT_RENDERABLE));
    }

    #[test]
    fn offscreen_device_limits_cover_renderer_layout_requirements() {
        let limits = required_render_limits(&wgpu::Limits {
            max_storage_buffers_per_shader_stage:
                HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            ..wgpu::Limits::default()
        });

        assert!(limits.max_bind_groups >= REQUIRED_RENDER_BIND_GROUP_LIMIT);
        assert!(limits.max_texture_array_layers >= REFLECTION_PROBE_REQUIRED_TEXTURE_ARRAY_LAYERS);
        assert!(
            limits.max_sampled_textures_per_shader_stage
                >= POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE
        );
        assert!(
            limits.max_storage_buffers_per_shader_stage
                >= HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
        assert!(
            limits.max_storage_buffers_per_shader_stage
                >= MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
    }

    #[test]
    fn offscreen_device_limits_keep_hzb_occlusion_optional_when_only_mesh_capacity_exists() {
        let limits = required_render_limits(&wgpu::Limits {
            max_storage_buffers_per_shader_stage:
                MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            ..wgpu::Limits::default()
        });

        assert!(
            limits.max_storage_buffers_per_shader_stage
                >= MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
        assert!(
            limits.max_storage_buffers_per_shader_stage
                < HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
    }

    #[test]
    fn offscreen_device_limits_keep_extra_storage_buffers_optional_when_adapter_limit_is_lower() {
        let limits = required_render_limits(&wgpu::Limits::default());

        assert!(
            limits.max_storage_buffers_per_shader_stage
                < MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
    }
}
