use crate::core::math::UVec2;

use super::{
    RenderCameraTargetWritebackReport, RenderCapabilityClass, RenderCapabilityKind,
    RenderCapabilityMismatchDetail, RenderCapabilitySummary, RenderGraphExecutionCoverageReport,
    RenderGraphStageExecutionReport, RenderHistoryCopyReport, RenderQualityProfile,
};
use crate::core::framework::render::TaaQualityPreset;

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
        ]
    );
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
