use zr_rhi::{RenderDeviceFeature, RenderDeviceFeatureSet};

use super::{device_request_policy_for_startup_options, SceneRendererStartupOptions};

const OUTER_CONSTRUCT_SOURCE: &str = include_str!("../new_with_icon_source.rs");
const CORE_CONSTRUCT_SOURCE: &str =
    include_str!("../../scene_renderer_core_construct/construct/construct.rs");
const MESH_CONSTRUCT_SOURCE: &str = include_str!("../../../mesh/mesh_pipeline_cache/construct.rs");
const UI_CONSTRUCT_SOURCE: &str = include_str!("../../../ui/construct.rs");

fn supported_features(features: &[RenderDeviceFeature]) -> RenderDeviceFeatureSet {
    let mut supported = RenderDeviceFeatureSet::default();
    for feature in features {
        supported.insert(*feature);
    }
    supported
}

#[test]
fn startup_gpu_timing_requests_optional_query_features_before_device_creation() {
    let supported = supported_features(&[
        RenderDeviceFeature::GpuTimestamp,
        RenderDeviceFeature::PipelineStatistics,
    ]);
    let timing_policy = device_request_policy_for_startup_options(
        SceneRendererStartupOptions::default().with_gpu_timing(),
    );
    let timing_negotiation = timing_policy
        .negotiate(&supported)
        .expect("optional query features must not reject a supported adapter");

    assert!(timing_negotiation
        .requested_features()
        .contains(RenderDeviceFeature::GpuTimestamp));
    assert!(timing_negotiation
        .requested_features()
        .contains(RenderDeviceFeature::PipelineStatistics));
}

#[test]
fn default_startup_leaves_optional_query_features_unrequested() {
    let supported = supported_features(&[
        RenderDeviceFeature::GpuTimestamp,
        RenderDeviceFeature::PipelineStatistics,
    ]);
    let negotiation =
        device_request_policy_for_startup_options(SceneRendererStartupOptions::default())
            .negotiate(&supported)
            .expect("the baseline profile must negotiate on every adapter");

    assert!(negotiation.requested_features().is_empty());
    assert!(negotiation.unavailable_features().is_empty());
}

#[test]
fn product_renderer_bootstrap_does_not_borrow_raw_queue() {
    let outer_product = OUTER_CONSTRUCT_SOURCE
        .split_once("#[cfg(test)]")
        .map(|(product, _)| product)
        .expect("renderer construct should retain a test-module boundary");
    let mesh_product = MESH_CONSTRUCT_SOURCE
        .split_once("pub(crate) fn new_with_adapter_facts")
        .map(|(_, product)| product)
        .and_then(|product| product.split_once("fn oit_storage_entry"))
        .map(|(product, _)| product)
        .expect("mesh product constructor should remain bounded");
    let ui_product = UI_CONSTRUCT_SOURCE
        .split_once("pub(crate) fn new_with_font_collection")
        .map(|(_, product)| product)
        .expect("UI product constructor should remain present");

    assert!(!outer_product.contains("backend.queue"));
    assert!(!CORE_CONSTRUCT_SOURCE.contains("wgpu::Queue"));
    assert!(!mesh_product.contains("wgpu::Queue"));
    assert!(!ui_product.contains("wgpu::Queue"));
}
