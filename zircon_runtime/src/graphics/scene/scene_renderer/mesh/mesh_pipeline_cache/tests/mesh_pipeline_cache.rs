use crate::core::framework::render::ShaderPassType;
use crate::graphics::scene::resources::default_pipeline_key;

use super::super::pipeline_creation_metrics::MeshPipelineCreationMetrics;
use super::{
    duration_microseconds_saturating, MeshPassPipelineKind, MeshPipelineCache,
    MeshPipelineVariantId, PipelineAdmissionKey, PipelineCreationTarget, ShaderSourceValidationKey,
};

#[test]
fn pipeline_admission_key_distinguishes_base_and_oit_for_the_same_variant() {
    let variant_id = MeshPipelineVariantId::new(17);
    let base = PipelineAdmissionKey::new(
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
        variant_id,
    );
    let oit = PipelineAdmissionKey::new(PipelineCreationTarget::Oit, variant_id);

    assert_ne!(base, oit);
    let mut targets = std::collections::HashSet::new();
    targets.insert(base);
    targets.insert(oit);
    assert_eq!(targets.len(), 2);
}

#[test]
fn pipeline_creation_duration_conversion_saturates_at_u64_microseconds() {
    assert_eq!(
        duration_microseconds_saturating(std::time::Duration::from_micros(17)),
        17
    );
    assert_eq!(
        duration_microseconds_saturating(std::time::Duration::from_secs(u64::MAX)),
        u64::MAX
    );
}

#[test]
fn pipeline_creation_metrics_accumulate_creation_calls_and_cpu_time() {
    let metrics = MeshPipelineCreationMetrics::default();
    let target = PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base);
    metrics.record_render_pipeline_creation(target, std::time::Duration::from_micros(23));
    metrics.record_shader_module_creation(target, std::time::Duration::from_micros(17));
    metrics.record_async_base_pipeline_queue_wait(std::time::Duration::from_micros(11));

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.render_pipeline_creation_count, 1);
    assert_eq!(snapshot.shader_module_creation_count, 1);
    assert_eq!(snapshot.render_pipeline_creation_cpu_microseconds, 23);
    assert_eq!(snapshot.shader_module_creation_cpu_microseconds, 17);
    assert_eq!(snapshot.async_base_pipeline_queue_wait_count, 1);
    assert_eq!(snapshot.async_base_pipeline_queue_wait_microseconds, 11);
}

#[test]
fn mesh_pipeline_cache_is_send_for_render_framework_state() {
    fn assert_send<T: Send>() {}

    assert_send::<MeshPipelineCache>();
}

fn assert_cache_hit_precedes_variant_projection(
    source: &str,
    function_name: &str,
    cache_lookup: &str,
) {
    let function = source
        .split_once(function_name)
        .map(|(_, function)| function)
        .expect("ensure function must exist");
    let cache_lookup = function
        .find(cache_lookup)
        .expect("ensure function must check its pipeline cache");
    let variant_projection = function
        .find("pipeline_and_shader_key_for_variant")
        .expect("ensure function must project its variant on a cache miss");

    assert!(
        cache_lookup < variant_projection,
        "pipeline cache hit must return before variant cloning and shader source assembly"
    );
}

#[test]
fn mesh_pipeline_cache_hits_precede_variant_and_shader_projection() {
    let cases = [
        (
            include_str!("../ensure_pipeline.rs"),
            "ensure_pipeline_admission_for_variant",
            // BUG: [CR-R02-runtime_wave12_graphics_mesh_pipeline-0004] Base 通过 base_pipeline_is_ready 查缓存；切片却在后续创建分支命中此字面量，晚于变体投影，顺序断言必失败。
            "mesh_variant_pipelines",
        ),
        (
            include_str!("../ensure_oit_pipeline.rs"),
            "ensure_oit_pipeline_admission_for_base_variant",
            "oit_mesh_variant_pipelines",
        ),
        (
            include_str!("../ensure_gbuffer_pipeline.rs"),
            "ensure_gbuffer_pipeline_admission_for_variant",
            "gbuffer_mesh_pipelines",
        ),
        (
            include_str!("../ensure_depth_prepass_pipeline.rs"),
            "ensure_depth_prepass_pipeline_admission_for_variant",
            "depth_prepass_mesh_pipelines",
        ),
        (
            include_str!("../ensure_shadow_pipeline.rs"),
            "ensure_shadow_pipeline_admission_for_variant",
            "shadow_mesh_pipelines",
        ),
        (
            include_str!("../ensure_velocity_pipeline.rs"),
            "ensure_velocity_pipeline_admission_for_variant",
            "velocity_mesh_pipelines",
        ),
        (
            include_str!("../ensure_taa_reactive_mask_pipeline.rs"),
            "ensure_taa_reactive_pipeline_admission_for_variant",
            "taa_reactive_pipeline_is_cached",
        ),
    ];

    for (source, function_name, cache_lookup) in cases {
        assert_cache_hit_precedes_variant_projection(source, function_name, cache_lookup);
    }
}

#[test]
fn gbuffer_pipeline_consumer_uses_typed_admission_without_frame_path_expect() {
    let cache = include_str!("../ensure_gbuffer_pipeline.rs");
    let consumer =
        include_str!("../../../deferred/deferred_scene_resources/record_gbuffer_geometry.rs");

    assert!(cache.contains("gbuffer_variant_admission_for_command_variant"));
    assert!(cache.contains("ensure_gbuffer_pipeline_admission_for_variant"));
    assert!(cache.contains("PipelineAdmission<()>"));
    assert!(cache.contains("gbuffer_pipeline_for_ready_variant"));
    assert!(consumer.contains("PipelineAdmission::Ready"));
    assert!(consumer.contains("record_pipeline_fallback_for_command_variant"));
    assert!(!consumer.contains("deferred GBuffer command must resolve a mesh pipeline"));
}

#[test]
fn shader_source_validation_key_distinguishes_hot_reloaded_source_identity() {
    let variant_key =
        default_pipeline_key().shader_variant_key(ShaderPassType::Forward, "wgpu-runtime");
    let previous = ShaderSourceValidationKey::new(&variant_key, "source-a|segment-a".to_string());
    let updated = ShaderSourceValidationKey::new(&variant_key, "source-b|segment-b".to_string());

    assert_ne!(previous, updated);
}
