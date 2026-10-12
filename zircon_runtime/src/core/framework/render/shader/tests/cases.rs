use zircon_runtime_interface::resource::ResourceId;

use crate::core::framework::render::{
    GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
    SHADING_MODEL_ID_STANDARD_PBR,
};

use super::{
    ShaderPipelineDiagnosticStage, ShaderPipelineFallbackAction, ShaderPipelineFallbackState,
    ShaderPipelineTarget, ShaderPipelineTargetMetrics, ShaderSourceValidationMetrics,
    ShaderVariantMissReport,
};

#[test]
fn shader_variant_miss_report_coalesces_pipeline_fallback_context_and_age() {
    let key = ShaderVariantKey {
        material_shader: ResourceId::from_stable_label("res://materials/fallback.zshader"),
        material_revision: 14,
        material_layout_hash: 0,
        material_option_bits: 0,
        geometry_source: GeometrySourceId::new(3),
        shading_model: SHADING_MODEL_ID_STANDARD_PBR,
        pass_type: ShaderPassType::Forward,
        features: ShaderFeatureBits::default(),
        quality: ShaderQualityTier::High,
        platform_token: "wgpu-runtime".to_string(),
    };
    let mut report = ShaderVariantMissReport::default();

    report.record_pipeline_fallback(
        &key,
        31,
        41,
        "base_scene_opaque",
        ShaderPipelineFallbackState::Deferred,
        ShaderPipelineFallbackAction::DeferDraw,
        "queue_saturated",
        7,
    );
    report.record_pipeline_fallback(
        &key,
        31,
        41,
        "base_scene_opaque",
        ShaderPipelineFallbackState::Deferred,
        ShaderPipelineFallbackAction::DeferDraw,
        "queue_saturated",
        19,
    );

    assert_eq!(report.pipeline_deferred_draw_count, 2);
    assert_eq!(report.pipeline_failed_draw_count, 0);
    let fallback = report
        .pipeline_fallbacks()
        .first()
        .expect("coalesced fallback diagnostic");
    assert_eq!(fallback.pipeline_variant_id, 31);
    assert_eq!(fallback.entity_id, 41);
    assert_eq!(fallback.consumer, "base_scene_opaque");
    assert_eq!(fallback.reason, "queue_saturated");
    assert_eq!(fallback.state_age_microseconds, 19);
    assert_eq!(fallback.occurrence_count, 2);
    assert!(fallback
        .variant_key
        .contains("res://materials/fallback.zshader"));
}

#[test]
fn shader_variant_miss_report_keeps_unresolved_variant_failures_visible() {
    let mut report = ShaderVariantMissReport::default();

    report.record_unresolved_pipeline_fallback(
        91,
        17,
        "base_scene_opaque",
        ShaderPipelineFallbackState::Failed,
        ShaderPipelineFallbackAction::RejectDraw,
        "unknown_variant",
        5,
    );
    report.record_unresolved_pipeline_fallback(
        91,
        17,
        "base_scene_opaque",
        ShaderPipelineFallbackState::Failed,
        ShaderPipelineFallbackAction::RejectDraw,
        "unknown_variant",
        13,
    );

    assert_eq!(report.pipeline_failed_draw_count, 2);
    let fallback = report
        .pipeline_fallbacks()
        .first()
        .expect("unresolved fallback diagnostic");
    assert_eq!(fallback.variant_key, "unresolved-pipeline-variant:91");
    assert_eq!(fallback.state_age_microseconds, 13);
    assert_eq!(fallback.occurrence_count, 2);
}

#[test]
fn shader_variant_miss_report_full_fallback_buffer_still_coalesces_existing_contexts() {
    let mut report = ShaderVariantMissReport::default();
    for index in 0..ShaderVariantMissReport::MAX_PIPELINE_FALLBACKS {
        report.record_unresolved_pipeline_fallback(
            index as u32,
            index as u64,
            "base_scene_opaque",
            ShaderPipelineFallbackState::Failed,
            ShaderPipelineFallbackAction::RejectDraw,
            "unknown_variant",
            5,
        );
    }

    report.record_unresolved_pipeline_fallback(
        u32::MAX,
        u64::MAX,
        "base_scene_opaque",
        ShaderPipelineFallbackState::Failed,
        ShaderPipelineFallbackAction::RejectDraw,
        "unknown_variant",
        7,
    );
    report.record_unresolved_pipeline_fallback(
        0,
        0,
        "base_scene_opaque",
        ShaderPipelineFallbackState::Failed,
        ShaderPipelineFallbackAction::RejectDraw,
        "unknown_variant",
        11,
    );

    assert_eq!(
        report.pipeline_fallbacks().len(),
        ShaderVariantMissReport::MAX_PIPELINE_FALLBACKS
    );
    assert_eq!(
        report.pipeline_failed_draw_count,
        ShaderVariantMissReport::MAX_PIPELINE_FALLBACKS + 2
    );
    let first = &report.pipeline_fallbacks()[0];
    assert_eq!(first.occurrence_count, 2);
    assert_eq!(first.state_age_microseconds, 11);
}

#[test]
fn shader_variant_miss_report_groups_runtime_outcomes_by_variant_dimensions() {
    let key = ShaderVariantKey {
        material_shader: ResourceId::from_stable_label("res://materials/runtime-hit.zshader"),
        material_revision: 11,
        material_layout_hash: 0,
        material_option_bits: 0,
        geometry_source: GeometrySourceId::new(3),
        shading_model: SHADING_MODEL_ID_STANDARD_PBR,
        pass_type: ShaderPassType::Velocity,
        features: ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST),
        quality: ShaderQualityTier::High,
        platform_token: "wgpu-runtime".to_string(),
    };
    let mut report = ShaderVariantMissReport::default();

    report.record_request(&key);
    report.record_disk_hit(&key);
    report.record_compile_miss(&key);
    report.record_disk_write(&key);
    report.record_disk_error(&key);

    assert_eq!(report.request_count, 1);
    assert_eq!(report.disk_hit_count, 1);
    assert_eq!(report.compile_miss_count, 1);
    assert_eq!(report.disk_write_count, 1);
    assert_eq!(report.disk_error_count, 1);

    let pass = report
        .dimension_summary
        .pass_types
        .get("velocity")
        .expect("velocity runtime dimension");
    assert_eq!(pass.request_count, 1);
    assert_eq!(pass.disk_hit_count, 1);
    assert_eq!(pass.compile_miss_count, 1);
    assert_eq!(pass.disk_write_count, 1);
    assert_eq!(pass.disk_error_count, 1);
    assert_eq!(
        report.dimension_summary.geometry_source_ids["3"].disk_hit_count,
        1
    );
    assert_eq!(
        report.dimension_summary.shading_model_ids["2"].compile_miss_count,
        1
    );
    assert_eq!(
        report.dimension_summary.quality_tiers["high"].disk_write_count,
        1
    );
}

#[test]
fn shader_variant_miss_report_deduplicates_and_bounds_pipeline_diagnostics() {
    let key = ShaderVariantKey {
        material_shader: ResourceId::from_stable_label("res://materials/diagnostic.zshader"),
        material_revision: 12,
        material_layout_hash: 0,
        material_option_bits: 0,
        geometry_source: GeometrySourceId::new(3),
        shading_model: SHADING_MODEL_ID_STANDARD_PBR,
        pass_type: ShaderPassType::Forward,
        features: ShaderFeatureBits::default(),
        quality: ShaderQualityTier::High,
        platform_token: "wgpu-runtime".to_string(),
    };
    let mut report = ShaderVariantMissReport::default();

    report.record_pipeline_diagnostic(
        &key,
        ShaderPipelineDiagnosticStage::SourceAssembly,
        "missing surface entry",
    );
    report.record_pipeline_diagnostic(
        &key,
        ShaderPipelineDiagnosticStage::SourceAssembly,
        "missing surface entry",
    );
    for index in 0..ShaderVariantMissReport::MAX_PIPELINE_DIAGNOSTICS {
        report.record_pipeline_diagnostic(
            &key,
            ShaderPipelineDiagnosticStage::WgslValidation,
            format!("validation failure {index}"),
        );
    }

    assert_eq!(
        report.pipeline_diagnostics().len(),
        ShaderVariantMissReport::MAX_PIPELINE_DIAGNOSTICS
    );
    assert_eq!(
        report.pipeline_diagnostics()[0].stage,
        ShaderPipelineDiagnosticStage::SourceAssembly
    );
    assert!(report.pipeline_diagnostics()[0]
        .variant_key
        .contains("res://materials/diagnostic.zshader"));
}

#[test]
fn shader_variant_miss_report_deserialization_enforces_pipeline_diagnostic_limits() {
    let mut document = serde_json::to_value(ShaderVariantMissReport::default())
        .expect("default report serializes");
    let diagnostics = (0..=ShaderVariantMissReport::MAX_PIPELINE_DIAGNOSTICS)
        .map(|index| {
            serde_json::json!({
                "variant_key": format!("variant-{index}"),
                "stage": "pipeline_creation",
                "message": "x".repeat(4096),
            })
        })
        .collect();
    document["pipeline_diagnostics"] = serde_json::Value::Array(diagnostics);

    let report: ShaderVariantMissReport =
        serde_json::from_value(document).expect("diagnostics deserialize");

    assert_eq!(
        report.pipeline_diagnostics().len(),
        ShaderVariantMissReport::MAX_PIPELINE_DIAGNOSTICS
    );
    assert!(
        report.pipeline_diagnostics()[0].message.chars().count()
            <= super::MAX_PIPELINE_DIAGNOSTIC_MESSAGE_CHARS
    );
}

#[test]
fn shader_variant_miss_report_deserialization_enforces_pipeline_fallback_limits() {
    let mut document = serde_json::to_value(ShaderVariantMissReport::default())
        .expect("default report serializes");
    let fallbacks = (0..=ShaderVariantMissReport::MAX_PIPELINE_FALLBACKS)
        .map(|index| {
            serde_json::json!({
                "variant_key": format!("variant-{index}"),
                "pipeline_variant_id": index,
                "entity_id": index,
                "consumer": "base_scene_opaque",
                "state": "deferred",
                "action": "defer_draw",
                "reason": "compile_pending",
                "state_age_microseconds": index,
                "occurrence_count": 1,
            })
        })
        .collect();
    document["pipeline_fallbacks"] = serde_json::Value::Array(fallbacks);

    let report: ShaderVariantMissReport =
        serde_json::from_value(document).expect("fallback diagnostics deserialize");

    assert_eq!(
        report.pipeline_fallbacks().len(),
        ShaderVariantMissReport::MAX_PIPELINE_FALLBACKS
    );
}

#[test]
fn shader_variant_miss_report_legacy_json_defaults_pipeline_shape_gauges() {
    let mut document = serde_json::to_value(ShaderVariantMissReport::default())
        .expect("default report serializes");
    let object = document
        .as_object_mut()
        .expect("shader variant report serializes as an object");
    for field in [
        "registered_pipeline_variant_count",
        "registered_shader_variant_count",
        "texture_presence_normalized_pipeline_variant_count",
        "texture_presence_equivalent_pipeline_variant_count",
        "cached_render_pipeline_count",
        "cached_shader_module_count",
        "render_pipeline_creation_count",
        "shader_module_creation_count",
        "render_pipeline_creation_cpu_microseconds",
        "shader_module_creation_cpu_microseconds",
        "async_base_pipeline_queue_wait_count",
        "async_base_pipeline_queue_wait_microseconds",
        "shader_source_validation_metrics",
        "pipeline_deferred_draw_count",
        "pipeline_failed_draw_count",
        "pipeline_target_metrics",
        "pipeline_fallbacks",
    ] {
        object.remove(field);
    }

    let report: ShaderVariantMissReport =
        serde_json::from_value(document).expect("legacy report deserializes");

    assert_eq!(report.registered_pipeline_variant_count, 0);
    assert_eq!(report.registered_shader_variant_count, 0);
    assert_eq!(report.texture_presence_normalized_pipeline_variant_count, 0);
    assert_eq!(report.texture_presence_equivalent_pipeline_variant_count, 0);
    assert_eq!(report.cached_render_pipeline_count, 0);
    assert_eq!(report.cached_shader_module_count, 0);
    assert_eq!(report.render_pipeline_creation_count, 0);
    assert_eq!(report.shader_module_creation_count, 0);
    assert_eq!(report.render_pipeline_creation_cpu_microseconds, 0);
    assert_eq!(report.shader_module_creation_cpu_microseconds, 0);
    assert_eq!(report.async_base_pipeline_queue_wait_count, 0);
    assert_eq!(report.async_base_pipeline_queue_wait_microseconds, 0);
    assert_eq!(
        report.shader_source_validation_metrics,
        ShaderSourceValidationMetrics::default()
    );
    assert_eq!(report.pipeline_deferred_draw_count, 0);
    assert_eq!(report.pipeline_failed_draw_count, 0);
    assert_eq!(
        report.pipeline_target_metrics(ShaderPipelineTarget::ShadowDepth),
        ShaderPipelineTargetMetrics::default()
    );
    assert!(report.pipeline_fallbacks().is_empty());
}

#[test]
fn shader_source_validation_accumulation_keeps_one_coherent_cumulative_snapshot() {
    let mut destination = ShaderVariantMissReport::default();
    destination.record_shader_source_validation_metrics(ShaderSourceValidationMetrics {
        queued_count: 4,
        job_count: 4,
        unique_source_count: 2,
        duplicate_job_count: 2,
        success_count: 4,
        queue_wait_microseconds: 400,
        validation_cpu_microseconds: 800,
        ..ShaderSourceValidationMetrics::default()
    });
    let mut source = ShaderVariantMissReport::default();
    source.record_shader_source_validation_metrics(ShaderSourceValidationMetrics {
        queued_count: 5,
        already_pending_count: 1,
        full_count: 2,
        worker_unavailable_count: 3,
        job_count: 5,
        unique_source_count: 3,
        duplicate_job_count: 2,
        success_count: 4,
        failure_count: 1,
        queue_wait_microseconds: 50,
        validation_cpu_microseconds: 90,
    });

    destination.accumulate(source);

    assert_eq!(
        destination.shader_source_validation_metrics,
        ShaderSourceValidationMetrics {
            queued_count: 5,
            already_pending_count: 1,
            full_count: 2,
            worker_unavailable_count: 3,
            job_count: 5,
            unique_source_count: 3,
            duplicate_job_count: 2,
            success_count: 4,
            failure_count: 1,
            queue_wait_microseconds: 50,
            validation_cpu_microseconds: 90,
        }
    );
}

#[test]
fn shader_pipeline_target_metrics_keep_fixed_identity_and_roundtrip() {
    let expected = [
        (ShaderPipelineTarget::Base, "base"),
        (ShaderPipelineTarget::GBuffer, "gbuffer"),
        (ShaderPipelineTarget::DepthPrepass, "depth_prepass"),
        (ShaderPipelineTarget::HitProxy, "hit_proxy"),
        (ShaderPipelineTarget::ShadowDepth, "shadow_depth"),
        (
            ShaderPipelineTarget::ShadowDepthAlphaMask,
            "shadow_depth_alpha_mask",
        ),
        (ShaderPipelineTarget::Velocity, "velocity"),
        (ShaderPipelineTarget::TaaReactiveMask, "taa_reactive_mask"),
        (
            ShaderPipelineTarget::TaaReactiveMaterialMask,
            "taa_reactive_material_mask",
        ),
        (ShaderPipelineTarget::Oit, "oit"),
    ];
    assert_eq!(ShaderPipelineTarget::ALL.len(), expected.len());
    for (index, (target, token)) in expected.into_iter().enumerate() {
        assert_eq!(ShaderPipelineTarget::ALL[index], target);
        assert_eq!(target.index(), index);
        assert_eq!(target.token(), token);
    }

    let mut report = ShaderVariantMissReport::default();
    report.record_registered_pipeline_target_variant_count(ShaderPipelineTarget::ShadowDepth, 7);
    report.record_pipeline_target_runtime_metrics(
        ShaderPipelineTarget::ShadowDepth,
        ShaderPipelineTargetMetrics {
            unique_shader_source_count: 3,
            render_pipeline_creation_count: 5,
            shader_module_creation_count: 4,
            render_pipeline_creation_cpu_microseconds: 41,
            shader_module_creation_cpu_microseconds: 29,
            ..ShaderPipelineTargetMetrics::default()
        },
    );

    let json = serde_json::to_value(&report).expect("target metrics serialize");
    let roundtrip: ShaderVariantMissReport =
        serde_json::from_value(json).expect("target metrics deserialize");
    assert_eq!(
        roundtrip.pipeline_target_metrics(ShaderPipelineTarget::ShadowDepth),
        ShaderPipelineTargetMetrics {
            registered_pipeline_variant_count: 7,
            unique_shader_source_count: 3,
            render_pipeline_creation_count: 5,
            shader_module_creation_count: 4,
            render_pipeline_creation_cpu_microseconds: 41,
            shader_module_creation_cpu_microseconds: 29,
        }
    );
}

#[test]
fn shader_pipeline_target_accumulation_keeps_monotonic_coherent_snapshots() {
    let target = ShaderPipelineTarget::ShadowDepthAlphaMask;
    let mut destination = ShaderVariantMissReport::default();
    destination.record_registered_pipeline_target_variant_count(target, 4);
    destination.record_pipeline_target_runtime_metrics(
        target,
        ShaderPipelineTargetMetrics {
            unique_shader_source_count: 2,
            render_pipeline_creation_count: 4,
            shader_module_creation_count: 3,
            render_pipeline_creation_cpu_microseconds: 400,
            shader_module_creation_cpu_microseconds: 300,
            ..ShaderPipelineTargetMetrics::default()
        },
    );
    let mut source = ShaderVariantMissReport::default();
    source.record_registered_pipeline_target_variant_count(target, 6);
    source.record_pipeline_target_runtime_metrics(
        target,
        ShaderPipelineTargetMetrics {
            unique_shader_source_count: 5,
            render_pipeline_creation_count: 5,
            shader_module_creation_count: 4,
            render_pipeline_creation_cpu_microseconds: 50,
            shader_module_creation_cpu_microseconds: 40,
            ..ShaderPipelineTargetMetrics::default()
        },
    );

    destination.accumulate(source);

    assert_eq!(
        destination.pipeline_target_metrics(target),
        ShaderPipelineTargetMetrics {
            registered_pipeline_variant_count: 6,
            unique_shader_source_count: 5,
            render_pipeline_creation_count: 5,
            shader_module_creation_count: 4,
            render_pipeline_creation_cpu_microseconds: 50,
            shader_module_creation_cpu_microseconds: 40,
        }
    );
}

#[test]
fn shader_variant_miss_report_accumulation_normalizes_foreign_diagnostics() {
    let mut destination = ShaderVariantMissReport::default();
    let mut source = ShaderVariantMissReport::default();
    source
        .pipeline_diagnostics
        .push(super::ShaderPipelineDiagnostic {
            variant_key: "foreign-variant".to_string(),
            stage: ShaderPipelineDiagnosticStage::PipelineCreation,
            message: "x".repeat(4096),
        });

    destination.accumulate(source);

    assert_eq!(destination.pipeline_diagnostics().len(), 1);
    assert!(
        destination.pipeline_diagnostics()[0]
            .message
            .chars()
            .count()
            <= super::MAX_PIPELINE_DIAGNOSTIC_MESSAGE_CHARS
    );
}

#[test]
fn shader_variant_miss_report_registered_variant_gauges_use_latest_counts_and_max_accumulation() {
    let mut destination = ShaderVariantMissReport::default();
    destination.record_registered_variant_counts(16, 1, 1);
    destination.record_cached_gpu_object_counts(12, 3);
    assert_eq!(destination.registered_pipeline_variant_count, 16);
    assert_eq!(destination.registered_shader_variant_count, 1);
    assert_eq!(
        destination.texture_presence_normalized_pipeline_variant_count,
        1
    );
    assert_eq!(
        destination.texture_presence_equivalent_pipeline_variant_count,
        15
    );
    assert_eq!(destination.cached_render_pipeline_count, 12);
    assert_eq!(destination.cached_shader_module_count, 3);

    destination.record_registered_variant_counts(2, 2, 2);
    assert_eq!(destination.registered_pipeline_variant_count, 2);
    assert_eq!(destination.registered_shader_variant_count, 2);
    assert_eq!(
        destination.texture_presence_normalized_pipeline_variant_count,
        2
    );
    assert_eq!(
        destination.texture_presence_equivalent_pipeline_variant_count,
        0
    );

    let mut source = ShaderVariantMissReport::default();
    source.record_registered_variant_counts(8, 3, 4);
    source.record_cached_gpu_object_counts(6, 2);
    source.record_gpu_object_creation_totals(9, 4, 42, 17);
    source.record_async_base_pipeline_queue_wait_totals(3, 88);
    destination.accumulate(source);

    assert_eq!(destination.registered_pipeline_variant_count, 8);
    assert_eq!(destination.registered_shader_variant_count, 3);
    assert_eq!(
        destination.texture_presence_normalized_pipeline_variant_count,
        4
    );
    assert_eq!(
        destination.texture_presence_equivalent_pipeline_variant_count,
        4
    );
    assert_eq!(destination.cached_render_pipeline_count, 12);
    assert_eq!(destination.cached_shader_module_count, 3);
    assert_eq!(destination.render_pipeline_creation_count, 9);
    assert_eq!(destination.shader_module_creation_count, 4);
    assert_eq!(destination.render_pipeline_creation_cpu_microseconds, 42);
    assert_eq!(destination.shader_module_creation_cpu_microseconds, 17);
    assert_eq!(destination.async_base_pipeline_queue_wait_count, 3);
    assert_eq!(destination.async_base_pipeline_queue_wait_microseconds, 88);
}

#[test]
fn shader_variant_creation_accumulation_keeps_coherent_count_time_snapshots() {
    let mut destination = ShaderVariantMissReport::default();
    destination.record_gpu_object_creation_totals(4, 2, 400, 200);
    let mut more_creations = ShaderVariantMissReport::default();
    more_creations.record_gpu_object_creation_totals(5, 3, 50, 30);
    destination.accumulate(more_creations);

    assert_eq!(destination.render_pipeline_creation_count, 5);
    assert_eq!(destination.render_pipeline_creation_cpu_microseconds, 50);
    assert_eq!(destination.shader_module_creation_count, 3);
    assert_eq!(destination.shader_module_creation_cpu_microseconds, 30);

    let mut same_count_more_time = ShaderVariantMissReport::default();
    same_count_more_time.record_gpu_object_creation_totals(5, 3, 70, 40);
    same_count_more_time.record_async_base_pipeline_queue_wait_totals(2, 90);
    destination.accumulate(same_count_more_time);

    assert_eq!(destination.render_pipeline_creation_count, 5);
    assert_eq!(destination.render_pipeline_creation_cpu_microseconds, 70);
    assert_eq!(destination.shader_module_creation_count, 3);
    assert_eq!(destination.shader_module_creation_cpu_microseconds, 40);
    assert_eq!(destination.async_base_pipeline_queue_wait_count, 2);
    assert_eq!(destination.async_base_pipeline_queue_wait_microseconds, 90);

    let mut more_wait_samples_less_total = ShaderVariantMissReport::default();
    more_wait_samples_less_total.record_async_base_pipeline_queue_wait_totals(3, 30);
    destination.accumulate(more_wait_samples_less_total);

    assert_eq!(destination.async_base_pipeline_queue_wait_count, 3);
    assert_eq!(destination.async_base_pipeline_queue_wait_microseconds, 30);
}
