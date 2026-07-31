use crate::core::framework::render::{
    RenderBudgetKey, RenderColorLutReadbackReport, RenderExposureReadbackReport,
    RenderGraphExecutionResourceReport, RenderGraphPassProfileMetrics,
    RenderGraphStageExecutionReport, RenderHistoryCopyReport, RenderSceneVelocityReadbackReport,
};
use crate::core::math::UVec2;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::visibility::HzbOcclusionCullReport;
use crate::render_graph::{
    QueueLane, RenderGraphPassResourceAccess, RenderGraphResourceAccessKind,
    RenderGraphResourceKind, RenderPassId,
};

use super::{
    RenderGraphComputeDispatchRecord, RenderGraphExecutionRecord, RenderGraphLightGridReport,
};

#[test]
fn stage_execution_report_uses_fixed_stage_storage() {
    let source = include_str!("../render_graph_execution_record.rs");

    assert!(
        !source.contains("BTreeSet"),
        "per-frame stage diagnostics should not allocate a tree set"
    );
    assert!(
        source.contains("[false; RenderPassStage::ALL.len()]"),
        "stage diagnostics should use the fixed RenderPassStage domain"
    );
}

#[test]
fn compute_workload_audit_does_not_partition_dispatches_into_temporary_vectors() {
    let source = include_str!("../render_graph_execution_record.rs");

    assert!(
        !source.contains(".partition("),
        "per-pass compute workload audit should borrow the dispatch slice without temporary Vecs"
    );
    assert!(
        source.contains("first_matching_dispatch_index"),
        "the audit should retain the first matching dispatch by index"
    );
}

#[test]
fn execution_record_preserves_resource_binding_report() {
    let mut record = RenderGraphExecutionRecord::default();
    let report = RenderGraphExecutionResourceReport::new(6, 4, 2, 3);

    record.set_resource_report(report);

    assert_eq!(record.resource_report(), report);
}

#[test]
fn execution_record_preserves_history_copy_report() {
    let mut record = RenderGraphExecutionRecord::default();
    let report = RenderHistoryCopyReport::new(
        true,
        UVec2::new(640, 360),
        4,
        true,
        true,
        true,
        false,
        false,
        false,
        false,
    );

    record.set_history_copy_report(report);

    assert_eq!(record.history_copy_report(), report);
    assert!(record.history_copy_report().debug_marker_emitted);
}

#[test]
fn execution_record_preserves_scene_velocity_readback_report() {
    let mut record = RenderGraphExecutionRecord::default();
    let report = RenderSceneVelocityReadbackReport::from_raw_rg16_float_bytes(
        UVec2::new(2, 2),
        &[0, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0],
    );

    record.set_scene_velocity_readback_report(report);

    assert_eq!(record.scene_velocity_readback_report(), report);
    assert!(record.scene_velocity_readback_report().available);
    assert_eq!(
        record.scene_velocity_readback_report().nonzero_pixel_count,
        2
    );
}

#[test]
fn scene_velocity_readback_ignores_signed_zero_half_float_pixels() {
    let report = RenderSceneVelocityReadbackReport::from_raw_rg16_float_bytes(
        UVec2::new(2, 2),
        &[
            0, 0, 0, 0, // +0, +0
            0, 0x80, 0, 0, // -0, +0
            0, 0, 0, 0x80, // +0, -0
            1, 0, 0, 0, // smallest positive subnormal, +0
        ],
    );

    assert_eq!(report.nonzero_pixel_count, 1);
}

#[test]
fn execution_record_preserves_color_lut_readback_report() {
    let mut record = RenderGraphExecutionRecord::default();
    let report = RenderColorLutReadbackReport::from_raw_rgba16_float_identity_bytes(
        [1, 1, 1],
        &[0, 0, 0, 0, 0, 0, 0, 0x3c],
    );

    record.set_color_lut_readback_report(report);

    assert_eq!(record.color_lut_readback_report(), report);
    assert!(record.color_lut_readback_report().available);
    assert!(record.color_lut_readback_report().identity_within_epsilon());
}

#[test]
fn execution_record_preserves_exposure_readback_report() {
    let mut record = RenderGraphExecutionRecord::default();
    let report = RenderExposureReadbackReport::from_words([1.0, 9.7, 9.7, 1.0]);

    record.set_exposure_readback_report(report);

    assert_eq!(record.exposure_readback_report(), report);
    assert!(record.exposure_readback_report().available);
    assert!(record.exposure_readback_report().history_valid());
}

#[test]
fn execution_record_preserves_hzb_occlusion_cull_report() {
    let mut record = RenderGraphExecutionRecord::default();
    let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 2, 3, true);

    record.set_hzb_occlusion_cull_report(report);

    assert_eq!(record.hzb_occlusion_cull_report(), Some(report));
}

#[test]
fn execution_record_preserves_light_grid_report() {
    let mut record = RenderGraphExecutionRecord::default();
    let report = RenderGraphLightGridReport {
        light_count: 9,
        tile_count: 64,
        zbin_count: 32,
        non_empty_tile_count: 11,
        non_empty_zbin_count: 7,
        non_empty_cluster_count: 23,
        peak_lights_per_cluster: 5,
        average_lights_per_cluster_milli: 375,
    };

    record.set_light_grid_report(report);

    assert_eq!(record.light_grid_report(), Some(report));
}

#[test]
fn execution_record_counts_queue_lanes_from_executed_passes() {
    let mut record = RenderGraphExecutionRecord::default();

    record.push_executed_pass_with_declared_queue_and_resources(
        "cull",
        "virtual-geometry.node-cluster-cull",
        QueueLane::Graphics,
        QueueLane::AsyncCompute,
        Vec::new(),
    );
    record.push_executed_pass("main", "mesh.opaque", QueueLane::Graphics);

    assert_eq!(record.executed_queue_lane_count(QueueLane::AsyncCompute), 0);
    assert_eq!(record.executed_queue_lane_count(QueueLane::Graphics), 2);
    assert_eq!(record.executed_queue_lane_count(QueueLane::AsyncCopy), 0);
    assert_eq!(record.executed_queue_fallback_count(), 1);
}

#[test]
fn execution_record_preserves_executed_pass_resource_accesses() {
    let mut record = RenderGraphExecutionRecord::default();
    let resources = vec![
        RenderGraphPassResourceAccess {
            name: "scene-depth".to_string(),
            kind: RenderGraphResourceKind::TransientTexture,
            access: RenderGraphResourceAccessKind::Read,
            attachment_ops: None,
        },
        RenderGraphPassResourceAccess {
            name: "scene-color".to_string(),
            kind: RenderGraphResourceKind::TransientTexture,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: None,
        },
    ];

    record.push_executed_pass_with_resources(
        "opaque",
        "mesh.opaque",
        QueueLane::Graphics,
        resources.clone(),
    );

    assert_eq!(record.executed_pass_resources(), &[resources]);
    assert_eq!(record.executed_resource_access_count(), 2);
}

#[test]
fn execution_record_preserves_executed_pass_dependencies() {
    let mut record = RenderGraphExecutionRecord::default();
    let dependencies = vec![RenderPassId(2), RenderPassId(5)];

    record.push_executed_pass_with_declared_queue_dependencies_and_resources(
        "lighting",
        "lighting.light-grid",
        QueueLane::Graphics,
        QueueLane::Graphics,
        dependencies.clone(),
        Vec::new(),
    );

    assert_eq!(record.executed_pass_dependencies(), &[dependencies]);
    assert_eq!(record.executed_dependency_count(), 2);
}

#[test]
fn execution_record_keeps_post_process_nodes_out_of_render_graph_passes() {
    let mut record = RenderGraphExecutionRecord::default();

    record.push_executed_pass("overlay-gizmo", "overlay.gizmo", QueueLane::Graphics);
    record.push_executed_post_process_node("output-transfer");

    assert_eq!(record.executed_passes(), &["overlay-gizmo".to_string()]);
    assert_eq!(
        record.executed_post_process_nodes(),
        &["output-transfer".to_string()]
    );
    assert_eq!(record.executed_queue_lane_count(QueueLane::Graphics), 1);
}

#[test]
fn execution_record_preserves_renderer_stage_metadata() {
    let mut record = RenderGraphExecutionRecord::default();

    record.push_executed_pass("legacy-overlay", "overlay.legacy", QueueLane::Graphics);
    record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
        Some(RenderPassStage::Transparent3d),
        "particle-render",
        "particle.transparent",
        QueueLane::Graphics,
        QueueLane::Graphics,
        Vec::new(),
        Vec::new(),
    );
    record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
        Some(RenderPassStage::Transparent3d),
        "transparent-mesh",
        "mesh.transparent",
        QueueLane::Graphics,
        QueueLane::Graphics,
        Vec::new(),
        Vec::new(),
    );
    record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
        Some(RenderPassStage::PostProcess),
        "post-stack",
        "post.uber",
        QueueLane::Graphics,
        QueueLane::Graphics,
        Vec::new(),
        Vec::new(),
    );

    assert_eq!(
        record.executed_pass_stages(),
        &[
            None,
            Some(RenderPassStage::Transparent3d),
            Some(RenderPassStage::Transparent3d),
            Some(RenderPassStage::PostProcess),
        ]
    );
    assert_eq!(
        record.executed_stage_count(RenderPassStage::Transparent3d),
        2
    );
    assert_eq!(record.executed_stage_count(RenderPassStage::PostProcess), 1);
    assert_eq!(
        record.stage_execution_report(),
        RenderGraphStageExecutionReport::new(3, 1, 2, 1, 0)
    );
}

#[test]
fn execution_record_counts_named_resource_accesses() {
    let mut record = RenderGraphExecutionRecord::default();
    let shadow_write = RenderGraphPassResourceAccess {
        name: "shadow-atlas".to_string(),
        kind: crate::render_graph::RenderGraphResourceKind::External,
        access: RenderGraphResourceAccessKind::Write,
        attachment_ops: None,
    };
    let shadow_read = RenderGraphPassResourceAccess {
        name: "shadow-atlas".to_string(),
        kind: crate::render_graph::RenderGraphResourceKind::External,
        access: RenderGraphResourceAccessKind::Read,
        attachment_ops: None,
    };
    let scene_color_read = RenderGraphPassResourceAccess {
        name: "scene-color".to_string(),
        kind: crate::render_graph::RenderGraphResourceKind::External,
        access: RenderGraphResourceAccessKind::Read,
        attachment_ops: None,
    };

    record.push_executed_pass_with_resources(
        "shadow-atlas",
        "shadow.atlas",
        QueueLane::Graphics,
        vec![shadow_write],
    );
    record.push_executed_pass_with_resources(
        "opaque-mesh",
        "mesh.opaque",
        QueueLane::Graphics,
        vec![shadow_read, scene_color_read],
    );

    assert_eq!(
        record.executed_resource_access_count_for(
            "shadow-atlas",
            RenderGraphResourceAccessKind::Write,
        ),
        1
    );
    assert_eq!(
        record.executed_resource_access_count_for(
            "shadow-atlas",
            RenderGraphResourceAccessKind::Read
        ),
        1
    );
    assert_eq!(
        record.executed_resource_access_count_for(
            "scene-color",
            RenderGraphResourceAccessKind::Write,
        ),
        0
    );
}

#[test]
fn execution_record_counts_renderer_stage_order_violations() {
    let mut record = RenderGraphExecutionRecord::default();

    record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
        Some(RenderPassStage::PostProcess),
        "post-stack",
        "post.uber",
        QueueLane::Graphics,
        QueueLane::Graphics,
        Vec::new(),
        Vec::new(),
    );
    record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
        Some(RenderPassStage::Opaque3d),
        "late-opaque",
        "mesh.opaque",
        QueueLane::Graphics,
        QueueLane::Graphics,
        Vec::new(),
        Vec::new(),
    );
    record.push_executed_pass("legacy-gap", "legacy.gap", QueueLane::Graphics);
    record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
        Some(RenderPassStage::Shadow),
        "shadow-atlas",
        "shadow.atlas",
        QueueLane::Graphics,
        QueueLane::Graphics,
        Vec::new(),
        Vec::new(),
    );

    assert_eq!(
        record.stage_execution_report(),
        RenderGraphStageExecutionReport::new(3, 1, 3, 1, 1)
    );
}

#[test]
fn execution_record_preserves_pass_debug_markers() {
    let mut record = RenderGraphExecutionRecord::default();

    record.push_executed_pass_with_stage_declared_queue_dependencies_resources_and_debug_marker(
        Some(RenderPassStage::PostProcess),
        "clustered-lighting",
        "lighting.light-grid",
        QueueLane::Graphics,
        QueueLane::AsyncCompute,
        Vec::new(),
        Vec::new(),
        Some("zircon::RenderGraphPass::clustered-lighting".to_string()),
    );

    assert_eq!(
        record.executed_debug_markers(),
        &["zircon::RenderGraphPass::clustered-lighting".to_string()]
    );
    assert_eq!(record.executed_queue_fallback_count(), 1);
}

#[test]
fn profile_report_preserves_per_pass_compute_metrics() {
    let mut record = RenderGraphExecutionRecord::default();
    let compute_dispatches = vec![
        RenderGraphComputeDispatchRecord::new(
            "ssao-evaluate",
            "ao.ssao-evaluate",
            "zircon-ssao-pipeline",
            [8, 8, 1],
            [40, 30, 1],
            Vec::new(),
        )
        .with_uploaded_bytes(128),
        RenderGraphComputeDispatchRecord::new(
            "ssao-evaluate",
            "ao.ssao-evaluate",
            "zircon-ssao-blur-pipeline",
            [8, 8, 1],
            [40, 30, 1],
            Vec::new(),
        )
        .with_uploaded_bytes(64),
    ];
    record.push_pass_profile_with_budget_key_and_compute_dispatches(
        "ssao-evaluate",
        "ao.ssao-evaluate",
        RenderBudgetKey::Ssao,
        31,
        RenderGraphPassProfileMetrics::new(3, 0, 7),
        &compute_dispatches,
    );
    for dispatch in compute_dispatches {
        record.push_compute_dispatch(dispatch);
    }

    let profile = record.profile_report();

    assert_eq!(profile.pass_profiles.len(), 1);
    assert_eq!(profile.pass_profiles[0].draw_count, 3);
    assert_eq!(profile.pass_profiles[0].instance_count, 0);
    assert_eq!(profile.pass_profiles[0].state_change_count, 7);
    assert_eq!(profile.pass_profiles[0].dispatch_count, 2);
    assert_eq!(profile.pass_profiles[0].upload_bytes, 192);
}
