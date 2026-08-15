use crate::render_graph::{
    RenderGraphComputeDispatchExtent, RenderGraphComputeWorkload, RenderGraphPassResourceAccess,
    RenderGraphResourceAccessKind, RenderGraphResourceKind,
};

use super::super::RenderGraphExecutionRecord;
use super::{
    RenderGraphComputeDispatchRecord, RenderGraphComputeWorkloadAuditStatus,
    RenderGraphComputeWorkloadDispatchContext,
};

fn dispatch_context() -> RenderGraphComputeWorkloadDispatchContext {
    RenderGraphComputeWorkloadDispatchContext::new([40, 30], [1024, 1024], 130)
}

#[test]
fn froxel_dispatch_extent_tracks_quality_scaled_3d_and_xy_workloads() {
    let context = dispatch_context().with_froxel_grid_size([160, 90, 96]);
    let scatter = RenderGraphComputeWorkload::froxel_grid("scatter", [4, 4, 4]);
    let integrate = RenderGraphComputeWorkload::froxel_grid_xy("integrate", [8, 8, 1]);

    assert_eq!(
        context.expected_dispatch_groups(&scatter),
        Some([40, 23, 24])
    );
    assert_eq!(
        context.expected_dispatch_groups(&integrate),
        Some([20, 12, 1])
    );
}

#[test]
fn execution_record_tracks_compute_dispatch_metadata() {
    let mut record = RenderGraphExecutionRecord::default();

    record.push_compute_dispatch(RenderGraphComputeDispatchRecord::new(
        "ssao-evaluate",
        "compute.generic",
        "zircon-ssao-pipeline",
        [8, 8, 1],
        [40, 30, 1],
        vec!["ambient-occlusion".to_string()],
    ));
    record.push_compute_dispatch(
        RenderGraphComputeDispatchRecord::new(
            "light-grid-build",
            "lighting.light-grid",
            "zircon-cluster-pipeline",
            [8, 8, 1],
            [5, 4, 1],
            vec!["light-list".to_string()],
        )
        .with_resource_accesses(vec![RenderGraphPassResourceAccess {
            name: "light-list".to_string(),
            kind: RenderGraphResourceKind::TransientBuffer,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: None,
        }]),
    );

    assert_eq!(record.compute_dispatch_count(), 2);
    assert_eq!(record.compute_dispatch_group_volume_total(), 1220);
    assert_eq!(record.compute_storage_write_resource_count(), 2);
    assert_eq!(
        record.compute_dispatches()[0].storage_write_resources,
        ["ambient-occlusion".to_string()]
    );
    assert_eq!(record.compute_dispatches()[1].resource_accesses.len(), 1);
    assert_eq!(
        record.compute_dispatches()[1].resource_accesses[0].access,
        RenderGraphResourceAccessKind::Write
    );
}

#[test]
fn execution_record_aggregates_volumetric_compute_performance_metrics() {
    let mut record = RenderGraphExecutionRecord::default();
    for (pass, groups, uploaded_bytes) in [
        ("volumetric.media_inject", [40, 23, 24], 208),
        ("volumetric.light_scatter", [40, 23, 24], 288),
        ("volumetric.integrate", [20, 12, 1], 128),
    ] {
        record.push_compute_dispatch(
            RenderGraphComputeDispatchRecord::new(
                pass,
                pass,
                format!("zircon-{pass}"),
                [4, 4, 4],
                groups,
                Vec::new(),
            )
            .with_uploaded_bytes(uploaded_bytes),
        );
    }
    record.push_compute_dispatch(
        RenderGraphComputeDispatchRecord::new(
            "ssao-evaluate",
            "compute.generic",
            "zircon-ssao-pipeline",
            [8, 8, 1],
            [20, 12, 1],
            Vec::new(),
        )
        .with_uploaded_bytes(4_096),
    );

    assert_eq!(
        record.compute_dispatch_count_for_executor_prefix("volumetric."),
        3
    );
    assert_eq!(
        record.compute_dispatch_group_volume_total_for_executor_prefix("volumetric."),
        44_400
    );
    assert_eq!(
        record.compute_uploaded_bytes_total_for_executor_prefix("volumetric."),
        624
    );
}

#[test]
fn execution_record_audits_planned_compute_workloads_against_dispatches() {
    let mut record = RenderGraphExecutionRecord::default();
    let planned = RenderGraphComputeWorkload::new(
        "zircon-ssao-pipeline",
        [8, 8, 1],
        RenderGraphComputeDispatchExtent::PerPixel {
            target: "ambient-occlusion".to_string(),
            local_size: [8, 8],
        },
    );
    let matched = RenderGraphComputeDispatchRecord::new(
        "ssao-evaluate",
        "compute.generic",
        "zircon-ssao-pipeline",
        [8, 8, 1],
        [40, 30, 1],
        vec!["ambient-occlusion".to_string()],
    );
    let unexpected = RenderGraphComputeDispatchRecord::new(
        "unexpected-compute",
        "unexpected.executor",
        "unexpected-pipeline",
        [4, 4, 1],
        [1, 1, 1],
        Vec::new(),
    );

    record.audit_compute_workload(
        "ssao-evaluate",
        "compute.generic",
        Some(&planned),
        dispatch_context(),
        std::slice::from_ref(&matched),
    );
    record.audit_compute_workload(
        "compute-fixed",
        "compute.fixed",
        Some(&RenderGraphComputeWorkload::fixed(
            "fixed-pipeline",
            [4, 4, 1],
            [2, 3, 1],
        )),
        dispatch_context(),
        &[RenderGraphComputeDispatchRecord::new(
            "compute-fixed",
            "compute.fixed",
            "fixed-pipeline",
            [4, 4, 1],
            [2, 3, 1],
            Vec::new(),
        )],
    );
    record.audit_compute_workload(
        "hzb-build",
        "visibility.hzb-build",
        Some(&RenderGraphComputeWorkload::hzb_furthest(
            "zircon-hzb-build-pipeline",
            [8, 8, 1],
        )),
        dispatch_context(),
        &[RenderGraphComputeDispatchRecord::new(
            "hzb-build",
            "visibility.hzb-build",
            "zircon-hzb-build-pipeline",
            [8, 8, 1],
            [128, 128, 1],
            vec!["hzb-furthest".to_string()],
        )],
    );
    record.audit_compute_workload(
        "hzb-occlusion-cull",
        "visibility.hzb-occlusion-cull",
        Some(&RenderGraphComputeWorkload::indirect_args(
            "zircon-hzb-occlusion-cull-pipeline",
            [64, 1, 1],
        )),
        dispatch_context(),
        &[RenderGraphComputeDispatchRecord::new(
            "hzb-occlusion-cull",
            "visibility.hzb-occlusion-cull",
            "zircon-hzb-occlusion-cull-pipeline",
            [64, 1, 1],
            [3, 1, 1],
            vec!["mesh.indirect-args".to_string()],
        )],
    );
    record.audit_compute_workload(
        "light-grid-build",
        "lighting.light-grid",
        Some(&RenderGraphComputeWorkload::new(
            "zircon-cluster-pipeline",
            [8, 8, 1],
            RenderGraphComputeDispatchExtent::ClusterGrid,
        )),
        dispatch_context(),
        &[],
    );
    record.audit_compute_workload(
        "unexpected-compute",
        "unexpected.executor",
        None,
        dispatch_context(),
        &[unexpected],
    );

    assert_eq!(record.compute_workload_planned_count(), 5);
    assert_eq!(record.compute_workload_matched_count(), 4);
    assert_eq!(record.compute_workload_missing_dispatch_count(), 1);
    assert_eq!(record.compute_workload_unexpected_dispatch_count(), 1);
    assert_eq!(record.compute_workload_mismatch_count(), 0);
    assert_eq!(
        record.compute_workload_audit()[0].status,
        RenderGraphComputeWorkloadAuditStatus::Matched
    );
    assert_eq!(
        record.compute_workload_audit()[1].status,
        RenderGraphComputeWorkloadAuditStatus::Matched
    );
    assert_eq!(
        record.compute_workload_audit()[2].status,
        RenderGraphComputeWorkloadAuditStatus::Matched
    );
    assert_eq!(
        record.compute_workload_audit()[3].status,
        RenderGraphComputeWorkloadAuditStatus::Matched
    );
    assert_eq!(
        record.compute_workload_audit()[4].status,
        RenderGraphComputeWorkloadAuditStatus::MissingDispatch
    );
    assert_eq!(
        record.compute_workload_audit()[5].status,
        RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch
    );
    assert_eq!(
        record.compute_workload_audit()[0].planned_dispatch_groups,
        Some([40, 30, 1])
    );
    assert_eq!(
        record.compute_workload_audit()[1].planned_dispatch_groups,
        Some([2, 3, 1])
    );
    assert_eq!(
        record.compute_workload_audit()[2].planned_dispatch_groups,
        Some([128, 128, 1])
    );
    assert_eq!(
        record.compute_workload_audit()[3].planned_dispatch_groups,
        Some([3, 1, 1])
    );
    assert_eq!(
        record.compute_workload_audit()[4].planned_dispatch_groups,
        Some([5, 4, 1])
    );
    assert_eq!(
        record.compute_workload_audit()[5].actual_dispatch_groups,
        Some([1, 1, 1])
    );
}

#[test]
fn compute_workload_audit_preserves_matching_then_unexpected_dispatch_order() {
    let mut record = RenderGraphExecutionRecord::default();
    let planned = RenderGraphComputeWorkload::fixed("target-pipeline", [4, 4, 1], [1, 1, 1]);
    let dispatch = |pass_name: &str, executor_id: &str, pipeline_label: &str| {
        RenderGraphComputeDispatchRecord::new(
            pass_name,
            executor_id,
            pipeline_label,
            [4, 4, 1],
            [1, 1, 1],
            Vec::new(),
        )
    };
    let dispatches = [
        dispatch("foreign-before", "foreign.before", "foreign-pipeline"),
        dispatch("target", "target.executor", "target-pipeline"),
        dispatch("foreign-middle", "foreign.middle", "foreign-pipeline"),
        dispatch("target", "target.executor", "duplicate-a"),
        dispatch("target", "target.executor", "duplicate-b"),
        dispatch("foreign-after", "foreign.after", "foreign-pipeline"),
    ];

    record.audit_compute_workload(
        "target",
        "target.executor",
        Some(&planned),
        dispatch_context(),
        &dispatches,
    );

    assert_eq!(
        record
            .compute_workload_audit()
            .iter()
            .map(|audit| (audit.status, audit.pass_name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (RenderGraphComputeWorkloadAuditStatus::Matched, "target"),
            (
                RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch,
                "target"
            ),
            (
                RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch,
                "target"
            ),
            (
                RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch,
                "foreign-before"
            ),
            (
                RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch,
                "foreign-middle"
            ),
            (
                RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch,
                "foreign-after"
            ),
        ]
    );
}

#[test]
fn execution_record_audits_zero_indirect_arg_workload_as_zero_groups() {
    let mut record = RenderGraphExecutionRecord::default();
    let context = RenderGraphComputeWorkloadDispatchContext::new([40, 30], [1024, 1024], 0);

    record.audit_compute_workload(
        "hzb-occlusion-cull",
        "visibility.hzb-occlusion-cull",
        Some(&RenderGraphComputeWorkload::indirect_args(
            "zircon-hzb-occlusion-cull-pipeline",
            [64, 1, 1],
        )),
        context,
        &[RenderGraphComputeDispatchRecord::new(
            "hzb-occlusion-cull",
            "visibility.hzb-occlusion-cull",
            "zircon-hzb-occlusion-cull-pipeline",
            [64, 1, 1],
            [0, 1, 1],
            Vec::new(),
        )],
    );

    assert_eq!(
        record.compute_workload_audit()[0].planned_dispatch_groups,
        Some([0, 1, 1])
    );
    assert_eq!(
        record.compute_workload_audit()[0].status,
        RenderGraphComputeWorkloadAuditStatus::Matched
    );
}

#[test]
fn execution_record_audits_phase_local_indirect_arg_workload_groups() {
    let mut record = RenderGraphExecutionRecord::default();
    let context = RenderGraphComputeWorkloadDispatchContext::new([40, 30], [1024, 1024], 3)
        .with_indirect_args_dispatch_group_count(3);

    record.audit_compute_workload(
        "hzb-occlusion-cull",
        "visibility.hzb-occlusion-cull",
        Some(&RenderGraphComputeWorkload::indirect_args(
            "zircon-hzb-occlusion-cull-pipeline",
            [64, 1, 1],
        )),
        context,
        &[RenderGraphComputeDispatchRecord::new(
            "hzb-occlusion-cull",
            "visibility.hzb-occlusion-cull",
            "zircon-hzb-occlusion-cull-pipeline",
            [64, 1, 1],
            [3, 1, 1],
            Vec::new(),
        )],
    );

    assert_eq!(
        record.compute_workload_audit()[0].planned_dispatch_groups,
        Some([3, 1, 1])
    );
    assert_eq!(
        record.compute_workload_audit()[0].status,
        RenderGraphComputeWorkloadAuditStatus::Matched
    );
}

#[test]
fn execution_record_accepts_gpu_generated_indirect_group_count_without_cpu_readback() {
    let mut record = RenderGraphExecutionRecord::default();
    let context = RenderGraphComputeWorkloadDispatchContext::new([40, 30], [1024, 1024], 0);
    let actual = RenderGraphComputeDispatchRecord::new(
        "sss.scatter",
        "sss.scatter",
        "sss.scatter.burley",
        [8, 8, 1],
        [0, 1, 1],
        vec!["sss-scattered".to_string()],
    )
    .with_gpu_indirect_dispatch_groups();

    record.audit_compute_workload(
        "sss.scatter",
        "sss.scatter",
        Some(&RenderGraphComputeWorkload::indirect_args(
            "sss.scatter.burley",
            [8, 8, 1],
        )),
        context,
        &[actual],
    );

    assert_eq!(
        record.compute_workload_audit()[0].status,
        RenderGraphComputeWorkloadAuditStatus::Matched
    );
    assert_eq!(
        record.compute_workload_audit()[0].actual_dispatch_groups,
        None
    );
}

#[test]
fn resource_bound_dispatch_extents_keep_planned_groups_unknown_to_cpu_audit() {
    let context = dispatch_context();
    let from_buffer =
        RenderGraphComputeWorkload::from_buffer("indirect-pipeline", [8, 1, 1], "dispatch-args", 0);
    let per_pixel = RenderGraphComputeWorkload::per_pixel(
        "per-pixel-pipeline",
        [8, 8, 1],
        "output-color",
        [8, 8],
    );

    assert_eq!(context.expected_dispatch_groups(&from_buffer), None);
    assert_eq!(context.expected_dispatch_groups(&per_pixel), None);

    let mut record = RenderGraphExecutionRecord::default();
    record.audit_compute_workload(
        "indirect",
        "compute.indirect",
        Some(&from_buffer),
        context,
        &[RenderGraphComputeDispatchRecord::new(
            "indirect",
            "compute.indirect",
            "indirect-pipeline",
            [8, 1, 1],
            [0, 1, 1],
            Vec::new(),
        )
        .with_gpu_indirect_dispatch_groups()],
    );
    record.audit_compute_workload(
        "per-pixel",
        "compute.per-pixel",
        Some(&per_pixel),
        context,
        &[RenderGraphComputeDispatchRecord::new(
            "per-pixel",
            "compute.per-pixel",
            "per-pixel-pipeline",
            [8, 8, 1],
            [40, 30, 1],
            Vec::new(),
        )],
    );

    assert_eq!(record.compute_workload_matched_count(), 2);
    assert_eq!(record.compute_workload_mismatch_count(), 0);
    assert_eq!(
        record.compute_workload_audit()[0].planned_dispatch_groups,
        None
    );
    assert_eq!(
        record.compute_workload_audit()[0].actual_dispatch_groups,
        None
    );
    assert_eq!(
        record.compute_workload_audit()[1].planned_dispatch_groups,
        None
    );
    assert_eq!(
        record.compute_workload_audit()[1].actual_dispatch_groups,
        Some([40, 30, 1])
    );
}

#[test]
fn execution_record_flags_compute_workload_label_workgroup_and_extent_mismatches() {
    let mut record = RenderGraphExecutionRecord::default();
    let planned = RenderGraphComputeWorkload::new(
        "zircon-ssao-pipeline",
        [8, 8, 1],
        RenderGraphComputeDispatchExtent::Fixed([40, 30, 1]),
    );
    let wrong_label = RenderGraphComputeDispatchRecord::new(
        "ssao-evaluate",
        "compute.generic",
        "other-pipeline",
        [8, 8, 1],
        [40, 30, 1],
        Vec::new(),
    );
    let wrong_workgroup = RenderGraphComputeDispatchRecord::new(
        "ssao-evaluate-2",
        "compute.generic",
        "zircon-ssao-pipeline",
        [16, 8, 1],
        [40, 30, 1],
        Vec::new(),
    );
    let wrong_extent = RenderGraphComputeDispatchRecord::new(
        "ssao-evaluate-3",
        "compute.generic",
        "zircon-ssao-pipeline",
        [8, 8, 1],
        [39, 30, 1],
        Vec::new(),
    );

    record.audit_compute_workload(
        "ssao-evaluate",
        "compute.generic",
        Some(&planned),
        dispatch_context(),
        &[wrong_label],
    );
    record.audit_compute_workload(
        "ssao-evaluate-2",
        "compute.generic",
        Some(&planned),
        dispatch_context(),
        &[wrong_workgroup],
    );
    record.audit_compute_workload(
        "ssao-evaluate-3",
        "compute.generic",
        Some(&planned),
        dispatch_context(),
        &[wrong_extent],
    );

    assert_eq!(record.compute_workload_mismatch_count(), 3);
    assert_eq!(
        record.compute_workload_audit()[0].status,
        RenderGraphComputeWorkloadAuditStatus::PipelineLabelMismatch
    );
    assert_eq!(
        record.compute_workload_audit()[1].status,
        RenderGraphComputeWorkloadAuditStatus::WorkgroupSizeMismatch
    );
    assert_eq!(
        record.compute_workload_audit()[2].status,
        RenderGraphComputeWorkloadAuditStatus::DispatchExtentMismatch
    );
    assert_eq!(
        record.compute_workload_audit()[2].planned_dispatch_groups,
        Some([40, 30, 1])
    );
    assert_eq!(
        record.compute_workload_audit()[2].actual_dispatch_groups,
        Some([39, 30, 1])
    );
}
