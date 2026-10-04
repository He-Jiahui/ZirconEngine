use crate::core::math::UVec2;

use super::{
    normalize_texture_max_anisotropy, RenderCameraTargetWritebackReport, RenderCapabilityClass,
    RenderCapabilityKind, RenderCapabilityMismatchDetail, RenderCapabilitySummary,
    RenderGraphExecutionBatchReport, RenderGraphExecutionCoverageReport,
    RenderGraphStageExecutionReport, RenderHistoryCopyReport, RenderHistoryDomain,
    RenderHistoryDomainResetReason, RenderHistoryDomainStatus, RenderHistoryDomainsReport,
    RenderQualityProfile,
};
use crate::core::framework::render::{TaaQualityPreset, DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA};

#[test]
fn render_contract_root_exposes_graph_pass_profile_metrics() {
    let metrics = crate::core::framework::render::RenderGraphPassProfileMetrics::new(3, 5, 7);

    assert_eq!(metrics.draw_count, 3);
    assert_eq!(metrics.instance_count, 5);
    assert_eq!(metrics.state_change_count, 7);
}

#[test]
fn history_copy_report_counts_copied_slots_from_slot_flags() {
    let report = RenderHistoryCopyReport::new(
        true,
        UVec2::new(960, 540),
        6,
        true,
        true,
        false,
        true,
        true,
        true,
        false,
    );

    assert!(report.history_target_present);
    assert!(report.debug_marker_emitted);
    assert_eq!(report.target_size, UVec2::new(960, 540));
    assert_eq!(report.requested_copy_count, 6);
    assert_eq!(report.copied_count, 5);

    let missing_target_report = RenderHistoryCopyReport::new(
        false,
        UVec2::new(960, 540),
        1,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
    );
    assert!(!missing_target_report.debug_marker_emitted);
}

#[test]
fn history_domains_report_preserves_independent_committed_state() {
    let mut states = [RenderHistoryDomainStatus::default(); RenderHistoryDomain::COUNT];
    states[RenderHistoryDomain::TaaSceneColor.index()] =
        RenderHistoryDomainStatus::new(3, true, Some(41), None, None);
    states[RenderHistoryDomain::Exposure.index()] = RenderHistoryDomainStatus::new(
        8,
        false,
        Some(40),
        Some(RenderHistoryDomainResetReason::SourceUnavailable),
        Some(RenderHistoryDomainResetReason::SourceUnavailable),
    );

    let report = RenderHistoryDomainsReport::new(true, states);

    assert!(report.history_target_present);
    assert!(report.state(RenderHistoryDomain::TaaSceneColor).valid);
    assert_eq!(
        report
            .state(RenderHistoryDomain::TaaSceneColor)
            .last_successful_frame,
        Some(41)
    );
    assert!(!report.state(RenderHistoryDomain::Exposure).valid);
    assert_eq!(
        report
            .state(RenderHistoryDomain::Exposure)
            .active_reset_reason,
        Some(RenderHistoryDomainResetReason::SourceUnavailable)
    );
    assert_eq!(
        report
            .state(RenderHistoryDomain::Exposure)
            .frame_reset_reason,
        Some(RenderHistoryDomainResetReason::SourceUnavailable)
    );
    assert_eq!(RenderHistoryDomain::Exposure.label(), "exposure");
    assert_eq!(
        RenderHistoryDomainResetReason::StructuralCompatibilityChanged.diagnostic_code(),
        7
    );
}

#[test]
fn history_domain_contract_keeps_fixed_order_and_reset_codes() {
    assert_eq!(
        RenderHistoryDomain::ALL.map(RenderHistoryDomain::label),
        [
            "taa_scene_color",
            "hybrid_global_illumination",
            "ambient_occlusion",
            "screen_space_reflection",
            "hzb_furthest",
            "exposure",
            "volumetric_scattering",
        ]
    );
    assert_eq!(
        [
            RenderHistoryDomainResetReason::NeverProduced,
            RenderHistoryDomainResetReason::PreviousFrameUnavailable,
            RenderHistoryDomainResetReason::CameraCut,
            RenderHistoryDomainResetReason::AllocationChanged,
            RenderHistoryDomainResetReason::FeatureDisabled,
            RenderHistoryDomainResetReason::SourceUnavailable,
            RenderHistoryDomainResetReason::StructuralCompatibilityChanged,
        ]
        .map(RenderHistoryDomainResetReason::diagnostic_code),
        [1, 2, 3, 4, 5, 6, 7]
    );
}

#[test]
fn camera_target_writeback_report_separates_copy_and_conversion_debug_markers() {
    let size = UVec2::new(72, 40);
    let copied = RenderCameraTargetWritebackReport::copied(size);
    let ready = RenderCameraTargetWritebackReport::ready_for_copy(size);
    let converted = RenderCameraTargetWritebackReport::converted(size);
    let blocked = RenderCameraTargetWritebackReport::blocked_format_mismatch(size);

    assert!(copied.debug_marker_emitted);
    assert!(!copied.conversion_debug_marker_emitted);
    assert_eq!(copied.copied_count, 1);
    assert_eq!(converted.converted_count, 1);
    assert_eq!(converted.copied_count, 0);
    assert!(!converted.debug_marker_emitted);
    assert!(converted.conversion_debug_marker_emitted);
    assert!(!ready.debug_marker_emitted);
    assert!(!ready.conversion_debug_marker_emitted);
    assert!(!blocked.debug_marker_emitted);
    assert!(!blocked.conversion_debug_marker_emitted);
}

#[test]
fn graph_stage_execution_report_preserves_neutral_counts() {
    let report = RenderGraphStageExecutionReport::new(8, 2, 5, 4, 1);

    assert_eq!(report.staged_pass_count, 8);
    assert_eq!(report.unstaged_pass_count, 2);
    assert_eq!(report.unique_stage_count, 5);
    assert_eq!(report.stage_transition_count, 4);
    assert_eq!(report.stage_order_violation_count, 1);
}

#[test]
fn graph_execution_batch_report_preserves_neutral_counts() {
    let report = RenderGraphExecutionBatchReport::new(5, 18, 3, 1, 1, 7, 4);

    assert_eq!(report.planned_batch_count, 5);
    assert_eq!(report.planned_live_pass_count, 18);
    assert_eq!(report.graphics_batch_count, 3);
    assert_eq!(report.async_compute_batch_count, 1);
    assert_eq!(report.async_copy_batch_count, 1);
    assert_eq!(report.max_passes_per_batch, 7);
    assert_eq!(report.queue_transition_count, 4);
}

#[test]
fn graph_execution_coverage_report_preserves_neutral_counts() {
    let report = RenderGraphExecutionCoverageReport::new(14, 15, 13, 1, 2, 1);

    assert_eq!(report.planned_live_pass_count, 14);
    assert_eq!(report.executed_pass_count, 15);
    assert_eq!(report.matched_planned_pass_count, 13);
    assert_eq!(report.missing_planned_pass_count, 1);
    assert_eq!(report.unexpected_executed_pass_count, 2);
    assert_eq!(report.duplicate_executed_pass_count, 1);
}

#[test]
fn render_quality_profile_preserves_taa_quality_preset() {
    let profile = RenderQualityProfile::new("taa-high").with_taa_quality(TaaQualityPreset::High);

    assert_eq!(profile.taa_quality, TaaQualityPreset::High);
    assert_eq!(
        RenderQualityProfile::new("default").taa_quality,
        TaaQualityPreset::Medium
    );
}

#[test]
fn render_quality_profile_exposes_viewport_texture_mip_bias() {
    let profile = RenderQualityProfile::new("texture-budget").with_texture_mip_bias(2);

    assert_eq!(profile.texture_mip_bias, 2);
    assert_eq!(RenderQualityProfile::new("default").texture_mip_bias, 0);
}

#[test]
fn render_quality_profile_clamps_texture_anisotropy_to_supported_tiers() {
    let profile = RenderQualityProfile::new("texture-budget").with_texture_max_anisotropy(6);

    assert_eq!(profile.texture_max_anisotropy, 4);
    assert_eq!(
        RenderQualityProfile::new("default").texture_max_anisotropy,
        16
    );
    assert_eq!(normalize_texture_max_anisotropy(0), 1);
    assert_eq!(normalize_texture_max_anisotropy(2), 2);
    assert_eq!(normalize_texture_max_anisotropy(7), 4);
    assert_eq!(normalize_texture_max_anisotropy(255), 16);
}

#[test]
fn render_quality_profile_keeps_half_resolution_transparency_opt_in() {
    let profile = RenderQualityProfile::new("bandwidth-limited")
        .with_half_resolution_transparency(true)
        .with_half_resolution_transparency_depth_sigma(144);

    assert!(profile.features.half_resolution_transparency);
    assert_eq!(profile.half_resolution_transparency_depth_sigma, 144);
    assert!(
        !RenderQualityProfile::new("default")
            .features
            .half_resolution_transparency
    );
    assert_eq!(
        RenderQualityProfile::new("default").half_resolution_transparency_depth_sigma,
        DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA
    );
    assert_eq!(
        RenderQualityProfile::new("clamped")
            .with_half_resolution_transparency_depth_sigma(0)
            .half_resolution_transparency_depth_sigma,
        1
    );
}

#[test]
fn capability_class_report_splits_default_advanced_and_experimental_requirements() {
    let capabilities = RenderCapabilitySummary {
        backend_name: "class-report-test".to_string(),
        supports_fxaa: true,
        virtual_geometry_supported: true,
        supports_storage_buffers: true,
        supports_indirect_draw: true,
        supports_buffer_readback: true,
        acceleration_structures_supported: true,
        supports_buffer_binding_array: true,
        supports_texture_binding_array: true,
        ..RenderCapabilitySummary::default()
    };

    let default = capabilities.capability_class_report(RenderCapabilityClass::Default);
    assert_eq!(
        default.satisfied,
        vec![RenderCapabilityKind::ScreenSpaceAntiAlias]
    );
    assert!(default.missing.is_empty());

    let advanced = capabilities.capability_class_report(RenderCapabilityClass::Advanced);
    assert_eq!(
        advanced.satisfied,
        vec![
            RenderCapabilityKind::VirtualGeometry,
            RenderCapabilityKind::StorageBuffers,
            RenderCapabilityKind::IndirectDraw,
            RenderCapabilityKind::BufferReadback,
        ]
    );
    assert_eq!(
        advanced.missing,
        vec![
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::HybridGlobalIllumination,),
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::AsyncCompute),
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::AsyncCopy),
        ]
    );

    let experimental = capabilities.capability_class_report(RenderCapabilityClass::Experimental);
    assert_eq!(
        experimental.satisfied,
        vec![
            RenderCapabilityKind::AccelerationStructures,
            RenderCapabilityKind::BufferBindingArray,
            RenderCapabilityKind::TextureBindingArray,
        ]
    );
    assert_eq!(
        experimental.missing,
        vec![
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::InlineRayQuery),
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::RayTracingPipeline),
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::NonUniformResourceIndexing,),
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::PartiallyBoundBindingArray,),
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::NeuralCompute),
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::SparseTexture),
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::SubgroupOps),
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::PipelineStatisticsQuery),
        ]
    );
}

#[test]
fn bindless_material_capability_requires_all_three_texture_array_features() {
    let supported = RenderCapabilitySummary {
        supports_texture_binding_array: true,
        supports_partially_bound_binding_array: true,
        supports_non_uniform_resource_indexing: true,
        max_binding_array_elements_per_shader_stage: 2,
        max_binding_array_sampler_elements_per_shader_stage: 2,
        ..RenderCapabilitySummary::default()
    };
    assert!(supported.bindless_material_supported());

    for capabilities in [
        RenderCapabilitySummary {
            supports_partially_bound_binding_array: true,
            supports_non_uniform_resource_indexing: true,
            max_binding_array_elements_per_shader_stage: 2,
            max_binding_array_sampler_elements_per_shader_stage: 2,
            ..RenderCapabilitySummary::default()
        },
        RenderCapabilitySummary {
            supports_texture_binding_array: true,
            supports_non_uniform_resource_indexing: true,
            max_binding_array_elements_per_shader_stage: 2,
            max_binding_array_sampler_elements_per_shader_stage: 2,
            ..RenderCapabilitySummary::default()
        },
        RenderCapabilitySummary {
            supports_texture_binding_array: true,
            supports_partially_bound_binding_array: true,
            max_binding_array_elements_per_shader_stage: 2,
            max_binding_array_sampler_elements_per_shader_stage: 2,
            ..RenderCapabilitySummary::default()
        },
    ] {
        assert!(!capabilities.bindless_material_supported());
    }
}

#[test]
fn bindless_material_capability_requires_fallback_and_dynamic_slot_capacity() {
    let insufficient = RenderCapabilitySummary {
        supports_texture_binding_array: true,
        supports_partially_bound_binding_array: true,
        supports_non_uniform_resource_indexing: true,
        max_binding_array_elements_per_shader_stage: 4,
        max_binding_array_sampler_elements_per_shader_stage: 1,
        ..RenderCapabilitySummary::default()
    };

    assert_eq!(insufficient.bindless_material_slot_capacity(), 1);
    assert!(!insufficient.bindless_material_supported());
}

#[test]
fn capability_class_report_includes_subgroup_and_pipeline_statistics_gates() {
    let capabilities = RenderCapabilitySummary {
        supports_subgroup: true,
        supports_pipeline_statistics_query: true,
        ..RenderCapabilitySummary::default()
    };

    let experimental = capabilities.capability_class_report(RenderCapabilityClass::Experimental);

    assert!(experimental
        .satisfied
        .contains(&RenderCapabilityKind::SubgroupOps));
    assert!(experimental
        .satisfied
        .contains(&RenderCapabilityKind::PipelineStatisticsQuery));
}

#[test]
fn screen_space_anti_alias_capability_accepts_smaa() {
    let capabilities = RenderCapabilitySummary {
        supports_smaa: true,
        ..RenderCapabilitySummary::default()
    };

    let default = capabilities.capability_class_report(RenderCapabilityClass::Default);

    assert_eq!(
        default.satisfied,
        vec![RenderCapabilityKind::ScreenSpaceAntiAlias]
    );
    assert!(default.missing.is_empty());
}

#[test]
fn gpu_driven_submission_requires_indirect_multi_draw_and_first_instance() {
    let supported = RenderCapabilitySummary {
        supports_indirect_draw: true,
        supports_multi_draw_indirect: true,
        supports_indirect_first_instance: true,
        ..RenderCapabilitySummary::default()
    };
    assert!(supported.gpu_driven_submission_supported());

    for capabilities in [
        RenderCapabilitySummary {
            supports_multi_draw_indirect: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        },
        RenderCapabilitySummary {
            supports_indirect_draw: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        },
        RenderCapabilitySummary {
            supports_indirect_draw: true,
            supports_multi_draw_indirect: true,
            ..RenderCapabilitySummary::default()
        },
    ] {
        assert!(!capabilities.gpu_driven_submission_supported());
    }
}

#[test]
fn gpu_driven_indirect_count_requires_the_optional_count_feature() {
    let fixed_count_only = RenderCapabilitySummary {
        supports_indirect_draw: true,
        supports_multi_draw_indirect: true,
        supports_indirect_first_instance: true,
        ..RenderCapabilitySummary::default()
    };
    let count_enabled = RenderCapabilitySummary {
        supports_multi_draw_indirect_count: true,
        ..fixed_count_only.clone()
    };

    assert!(fixed_count_only.gpu_driven_submission_supported());
    assert!(!fixed_count_only.gpu_driven_indirect_count_supported());
    assert!(count_enabled.gpu_driven_indirect_count_supported());
}

#[test]
fn indirect_draw_submission_is_available_without_the_multi_draw_upgrade() {
    let per_draw = RenderCapabilitySummary {
        supports_indirect_draw: true,
        supports_indirect_first_instance: true,
        ..RenderCapabilitySummary::default()
    };

    assert!(per_draw.indirect_draw_submission_supported());
    assert!(!per_draw.gpu_driven_submission_supported());
}

#[test]
fn hzb_occlusion_culling_requires_storage_buffers_gpu_driven_and_binding_capacity() {
    const REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 10;

    let supported = RenderCapabilitySummary {
        supports_storage_buffers: true,
        max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
        supports_indirect_draw: true,
        supports_multi_draw_indirect: true,
        supports_indirect_first_instance: true,
        ..RenderCapabilitySummary::default()
    };
    assert!(supported.hzb_occlusion_culling_supported(REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE));

    for capabilities in [
        RenderCapabilitySummary {
            max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            supports_indirect_draw: true,
            supports_multi_draw_indirect: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        },
        RenderCapabilitySummary {
            supports_storage_buffers: true,
            max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            supports_multi_draw_indirect: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        },
        RenderCapabilitySummary {
            supports_storage_buffers: true,
            max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            supports_indirect_draw: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        },
        RenderCapabilitySummary {
            supports_storage_buffers: true,
            max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            supports_indirect_draw: true,
            supports_multi_draw_indirect: true,
            ..RenderCapabilitySummary::default()
        },
        RenderCapabilitySummary {
            supports_storage_buffers: true,
            max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE - 1,
            supports_indirect_draw: true,
            supports_multi_draw_indirect: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        },
    ] {
        assert!(!capabilities
            .hzb_occlusion_culling_supported(REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE));
    }
}
