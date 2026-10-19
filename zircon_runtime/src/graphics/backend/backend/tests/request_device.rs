use crate::graphics::resource_limits::{
    HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
    MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
    OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
    POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE,
    REFLECTION_PROBE_REQUIRED_TEXTURE_ARRAY_LAYERS,
};
use zr_rhi::{RenderDeviceFeature, RenderDeviceRequestPolicy};

use super::{
    required_render_features, required_render_limits, BINDLESS_MATERIAL_MAX_SLOT_COUNT,
    REQUIRED_RENDER_BIND_GROUP_LIMIT, WGPU_BINDLESS_MATERIAL_REQUIRED_FEATURES,
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
    let supported = required_render_features(zr_rhi_wgpu::GPU_TIMESTAMP_REQUIRED_FEATURES, &policy)
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
    assert!(requested
        .contains(wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,));
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
    assert!(limits.max_binding_array_elements_per_shader_stage <= BINDLESS_MATERIAL_MAX_SLOT_COUNT);
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

#[test]
fn device_request_error_preserves_typed_source_from_display_and_nested_chain() {
    use std::error::Error;

    use crate::graphics::GraphicsError;
    use zr_rhi::{
        RenderAdapterClass, RenderAdapterFacts, RenderBackendKind, RenderDeviceFeatureSet,
        RenderDeviceRequestFailure,
    };

    let adapter = RenderAdapterFacts::new(
        RenderBackendKind::Dx12,
        "typed-source-adapter",
        0x10de,
        0x2468,
        "typed-source-driver",
        RenderAdapterClass::Discrete,
        None,
        RenderDeviceFeatureSet::default(),
    );
    let negotiation = RenderDeviceRequestPolicy::mvp_baseline()
        .with_optional_feature(RenderDeviceFeature::GpuTimestamp)
        .negotiate(&RenderDeviceFeatureSet::default())
        .unwrap();
    let expected = RenderDeviceRequestFailure::new(
        adapter,
        negotiation,
        zr_rhi_wgpu::wgpu_device_limits(&wgpu::Limits::default()),
        "native request rejected",
    );
    let display = format!("render device request failed: {expected}");
    let debug = format!("DeviceRequest({expected:?})");
    let error: GraphicsError = expected.clone().into();
    assert_eq!(error.to_string(), display);
    assert_eq!(format!("{error:?}"), debug);

    let GraphicsError::DeviceRequest(payload) = &error else {
        panic!("From must retain the device-request variant");
    };
    let dynamic: &dyn Error = &error;
    let source = dynamic.source().unwrap();
    let typed = source.downcast_ref::<RenderDeviceRequestFailure>().unwrap();
    assert_eq!(typed, &expected);
    assert!(std::ptr::eq(typed, &**payload));
    assert!(source
        .downcast_ref::<Box<RenderDeviceRequestFailure>>()
        .is_none());
    assert!(source.source().is_none());
    assert_eq!(
        std::mem::size_of_val(payload),
        std::mem::size_of::<Box<RenderDeviceRequestFailure>>()
    );

    let nested = GraphicsError::FrameSubmissionSettlement {
        settlement: "original settlement diagnostic".to_owned(),
        source: Box::new(error),
    };
    let inner = nested
        .source()
        .unwrap()
        .downcast_ref::<Box<GraphicsError>>()
        .unwrap()
        .as_ref();
    assert_eq!(
        inner
            .source()
            .unwrap()
            .downcast_ref::<RenderDeviceRequestFailure>(),
        Some(&expected)
    );
    assert!(inner.source().unwrap().source().is_none());

    let GraphicsError::FrameSubmissionSettlement { source, .. } = nested else {
        panic!("the outer failure variant must remain unchanged");
    };
    let GraphicsError::DeviceRequest(payload) = *source else {
        panic!("the original typed device-request error must remain recoverable");
    };
    assert_eq!(payload.into_inner(), expected);
}
