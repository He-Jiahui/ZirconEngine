use crate::graphics::resource_limits::{
    HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
    MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
    OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
    POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE,
    REFLECTION_PROBE_REQUIRED_TEXTURE_ARRAY_LAYERS,
};
use crate::graphics::types::GraphicsError;
use zr_rhi::{RenderDeviceRequestFailure, RenderDeviceRequestPolicy};
use zr_rhi_wgpu::{
    WGPU_BINDLESS_MATERIAL_REQUIRED_FEATURES, WgpuDeviceRequest, wgpu_adapter_facts,
    wgpu_device_limits, wgpu_device_request,
};

const REQUIRED_RENDER_BIND_GROUP_LIMIT: u32 = 5;
const BINDLESS_MATERIAL_MIN_SLOT_COUNT: u32 = 2;
// Sampler-array limits are the narrower WGPU binding-array limit. Cap the initial material slab
// so the negotiated table remains portable when the renderer switches away from per-material
// texture groups.
const BINDLESS_MATERIAL_MAX_SLOT_COUNT: u32 = 1_000;
pub(super) struct RequestedDevice {
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    pub(super) profile_request: WgpuDeviceRequest,
}

pub(super) fn request_device(adapter: &wgpu::Adapter) -> Result<RequestedDevice, GraphicsError> {
    request_device_with_policy(adapter, &RenderDeviceRequestPolicy::mvp_baseline())
}

pub(super) fn request_device_with_policy(
    adapter: &wgpu::Adapter,
    policy: &RenderDeviceRequestPolicy,
) -> Result<RequestedDevice, GraphicsError> {
    let adapter_features = adapter.features();
    let profile_request = wgpu_device_request(adapter_features, policy)?;
    let requested_features = profile_request.requested_features();
    let requested_limits = required_render_limits(&adapter.limits(), requested_features);
    let adapter_facts = wgpu_adapter_facts(&adapter.get_info(), adapter_features);
    let failure_feature_negotiation = profile_request.feature_negotiation().clone();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-device"),
        required_features: requested_features,
        required_limits: requested_limits.clone(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .map_err(|error| {
        GraphicsError::DeviceRequest(RenderDeviceRequestFailure::new(
            adapter_facts,
            failure_feature_negotiation,
            wgpu_device_limits(&requested_limits),
            error.to_string(),
        ))
    })?;

    Ok(RequestedDevice {
        device,
        queue,
        profile_request,
    })
}

fn required_render_features(
    adapter_features: wgpu::Features,
    policy: &RenderDeviceRequestPolicy,
) -> Result<wgpu::Features, GraphicsError> {
    Ok(wgpu_device_request(adapter_features, policy)?.requested_features())
}

fn required_render_limits(
    adapter_limits: &wgpu::Limits,
    requested_features: wgpu::Features,
) -> wgpu::Limits {
    let mut limits = wgpu::Limits {
        max_bind_groups: REQUIRED_RENDER_BIND_GROUP_LIMIT,
        max_sampled_textures_per_shader_stage:
            POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE,
        max_texture_array_layers: REFLECTION_PROBE_REQUIRED_TEXTURE_ARRAY_LAYERS,
        ..wgpu::Limits::default()
    };
    let required_storage_buffers_per_shader_stage =
        HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
            .max(OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE);
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
    if requested_features.contains(WGPU_BINDLESS_MATERIAL_REQUIRED_FEATURES) {
        let binding_array_slot_count = adapter_limits
            .max_binding_array_elements_per_shader_stage
            .min(adapter_limits.max_binding_array_sampler_elements_per_shader_stage)
            .min(BINDLESS_MATERIAL_MAX_SLOT_COUNT);
        if binding_array_slot_count >= BINDLESS_MATERIAL_MIN_SLOT_COUNT {
            limits.max_binding_array_elements_per_shader_stage = binding_array_slot_count;
            limits.max_binding_array_sampler_elements_per_shader_stage = binding_array_slot_count;
        }
    }
    limits
}

#[cfg(test)]
mod tests {
    use crate::graphics::resource_limits::{
        HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
        MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
        OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
        POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE,
        REFLECTION_PROBE_REQUIRED_TEXTURE_ARRAY_LAYERS,
    };
    use zr_rhi::{RenderDeviceFeature, RenderDeviceRequestPolicy};

    use super::{
        BINDLESS_MATERIAL_MAX_SLOT_COUNT, REQUIRED_RENDER_BIND_GROUP_LIMIT,
        WGPU_BINDLESS_MATERIAL_REQUIRED_FEATURES, required_render_features, required_render_limits,
    };

    #[test]
    fn offscreen_device_baseline_does_not_require_rg11b10_render_target() {
        let features = required_render_features(
            wgpu::Features::RG11B10UFLOAT_RENDERABLE | wgpu::Features::INDIRECT_FIRST_INSTANCE,
            &RenderDeviceRequestPolicy::mvp_baseline(),
        )
        .expect("MVP negotiation must not fail");

        assert!(!features.contains(wgpu::Features::RG11B10UFLOAT_RENDERABLE));
        assert!(!features.contains(wgpu::Features::INDIRECT_FIRST_INSTANCE));
        assert!(!features.contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT));
    }

    #[test]
    fn offscreen_device_features_request_gpu_timestamps_only_when_fully_supported() {
        let policy = RenderDeviceRequestPolicy::mvp_baseline()
            .with_optional_feature(RenderDeviceFeature::GpuTimestamp);
        let supported =
            required_render_features(zr_rhi_wgpu::GPU_TIMESTAMP_REQUIRED_FEATURES, &policy)
                .expect("timestamp optional profile is supported");
        let partial = required_render_features(wgpu::Features::TIMESTAMP_QUERY, &policy)
            .expect("an unavailable optional profile must not reject the adapter");

        assert!(supported.contains(zr_rhi_wgpu::GPU_TIMESTAMP_REQUIRED_FEATURES));
        assert!(!partial.contains(wgpu::Features::TIMESTAMP_QUERY));
        assert!(!partial.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS));
    }

    #[test]
    fn offscreen_device_features_request_optional_capability_gates_only_when_supported() {
        let policy = RenderDeviceRequestPolicy::mvp_baseline()
            .with_optional_feature(RenderDeviceFeature::BindlessMaterialArrays)
            .with_optional_feature(RenderDeviceFeature::Subgroups)
            .with_optional_feature(RenderDeviceFeature::PipelineStatistics);
        let requested = required_render_features(
            wgpu::Features::TEXTURE_BINDING_ARRAY
                | wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY
                | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                | wgpu::Features::SUBGROUP
                | wgpu::Features::PIPELINE_STATISTICS_QUERY,
            &policy,
        )
        .expect("supported optional features must negotiate");
        let unavailable = required_render_features(wgpu::Features::empty(), &policy)
            .expect("unavailable optional features must not reject the adapter");

        assert!(requested.contains(wgpu::Features::TEXTURE_BINDING_ARRAY));
        assert!(requested.contains(wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY));
        assert!(requested.contains(
            wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
        ));
        assert!(requested.contains(wgpu::Features::SUBGROUP));
        assert!(requested.contains(wgpu::Features::PIPELINE_STATISTICS_QUERY));
        assert!(!unavailable.contains(wgpu::Features::TEXTURE_BINDING_ARRAY));
        assert!(!unavailable.contains(wgpu::Features::SUBGROUP));
        assert!(!unavailable.contains(wgpu::Features::PIPELINE_STATISTICS_QUERY));
    }

    #[test]
    fn offscreen_device_baseline_can_negotiate_an_empty_adapter_feature_set() {
        let features = required_render_features(
            wgpu::Features::empty(),
            &RenderDeviceRequestPolicy::mvp_baseline(),
        )
        .expect("MVP negotiation must not fail");

        assert!(features.is_empty());
    }

    #[test]
    fn offscreen_device_limits_cover_renderer_layout_requirements() {
        let limits = required_render_limits(
            &wgpu::Limits {
                max_storage_buffers_per_shader_stage:
                    HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
                        .max(OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE),
                ..wgpu::Limits::default()
            },
            wgpu::Features::empty(),
        );

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
        assert!(
            limits.max_storage_buffers_per_shader_stage
                >= OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
    }

    #[test]
    fn offscreen_device_limits_cover_oit_fragment_store_bindings() {
        let limits = required_render_limits(
            &wgpu::Limits {
                max_storage_buffers_per_shader_stage:
                    OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
                ..wgpu::Limits::default()
            },
            wgpu::Features::empty(),
        );

        assert_eq!(
            limits.max_storage_buffers_per_shader_stage,
            OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
    }

    #[test]
    fn offscreen_device_limits_keep_hzb_occlusion_optional_when_only_mesh_capacity_exists() {
        let limits = required_render_limits(
            &wgpu::Limits {
                max_storage_buffers_per_shader_stage:
                    MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
                ..wgpu::Limits::default()
            },
            wgpu::Features::empty(),
        );

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
        let limits = required_render_limits(&wgpu::Limits::default(), wgpu::Features::empty());

        assert!(
            limits.max_storage_buffers_per_shader_stage
                < MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
    }

    #[test]
    fn offscreen_device_limits_request_shared_binding_array_capacity_when_fully_supported() {
        let limits = required_render_limits(
            &wgpu::Limits {
                max_binding_array_elements_per_shader_stage: 4_096,
                max_binding_array_sampler_elements_per_shader_stage: 768,
                ..wgpu::Limits::default()
            },
            WGPU_BINDLESS_MATERIAL_REQUIRED_FEATURES,
        );

        assert_eq!(limits.max_binding_array_elements_per_shader_stage, 768);
        assert_eq!(
            limits.max_binding_array_sampler_elements_per_shader_stage,
            768
        );
        assert!(
            limits.max_binding_array_elements_per_shader_stage <= BINDLESS_MATERIAL_MAX_SLOT_COUNT
        );
    }

    #[test]
    fn offscreen_device_limits_do_not_request_binding_array_capacity_for_partial_features() {
        let limits = required_render_limits(
            &wgpu::Limits {
                max_binding_array_elements_per_shader_stage: 4_096,
                max_binding_array_sampler_elements_per_shader_stage: 768,
                ..wgpu::Limits::default()
            },
            wgpu::Features::TEXTURE_BINDING_ARRAY,
        );

        assert_eq!(limits.max_binding_array_elements_per_shader_stage, 0);
        assert_eq!(
            limits.max_binding_array_sampler_elements_per_shader_stage,
            0
        );
    }
}
