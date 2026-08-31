use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderAmbientOcclusionExecutionFailureFlags,
    RenderAmbientOcclusionExecutionStatus,
};
use crate::render_graph::{
    QueueLane, RenderGraphComputePipelineFallbackPolicy, RenderGraphComputePipelineFamily,
    RenderGraphComputePipelineResolution, RenderGraphPassResourceAccess,
    RenderGraphResourceAccessKind, RenderGraphResourceKind,
};

use super::ambient_occlusion::AmbientOcclusionExecutionContract;
use super::{RenderGraphComputeDispatchRecord, RenderGraphExecutionRecord};

fn texture_access(
    name: &str,
    access: RenderGraphResourceAccessKind,
) -> RenderGraphPassResourceAccess {
    RenderGraphPassResourceAccess {
        name: name.to_string(),
        kind: RenderGraphResourceKind::Texture,
        access,
        attachment_ops: None,
    }
}

fn record_complete_ao_work() -> RenderGraphExecutionRecord {
    let mut record = RenderGraphExecutionRecord::default();
    record.push_executed_pass_with_resources(
        "ssao-evaluate",
        "compute.generic",
        QueueLane::AsyncCompute,
        vec![texture_access(
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
            RenderGraphResourceAccessKind::Write,
        )],
    );
    record.push_compute_dispatch(
        RenderGraphComputeDispatchRecord::new(
            "ssao-evaluate",
            "compute.generic",
            "zircon-ssao-pipeline",
            [8, 8, 1],
            [20, 12, 1],
            vec![PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW.to_string()],
        )
        .with_pipeline_resolution(ready_pipeline_resolution("ambient-occlusion.evaluate", 41)),
    );
    record.push_executed_pass_with_resources(
        "ssao-spatial-denoise",
        "compute.generic",
        QueueLane::AsyncCompute,
        vec![
            texture_access(
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
                RenderGraphResourceAccessKind::Read,
            ),
            texture_access(
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
                RenderGraphResourceAccessKind::Write,
            ),
        ],
    );
    record.push_compute_dispatch(
        RenderGraphComputeDispatchRecord::new(
            "ssao-spatial-denoise",
            "compute.generic",
            "zircon-ssao-spatial-denoise-pipeline",
            [8, 8, 1],
            [20, 12, 1],
            vec![PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string()],
        )
        .with_pipeline_resolution(ready_pipeline_resolution(
            "ambient-occlusion.spatial-denoise",
            42,
        )),
    );
    record.push_executed_pass_with_resources(
        "deferred-lighting",
        "lighting.deferred",
        QueueLane::Graphics,
        vec![texture_access(
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            RenderGraphResourceAccessKind::Read,
        )],
    );
    record
}

fn ready_pipeline_resolution(
    family: &str,
    artifact_fingerprint: u64,
) -> RenderGraphComputePipelineResolution {
    RenderGraphComputePipelineResolution::ready(
        &RenderGraphComputePipelineFallbackPolicy::last_good(family, 3),
        artifact_fingerprint,
        Some((7, 3)),
    )
}

fn enabled_contract() -> AmbientOcclusionExecutionContract {
    AmbientOcclusionExecutionContract::enabled(2, 2, 3, 1, 41, 41, "ssao-spatial-denoise")
}

#[test]
fn disabled_ao_receipt_ignores_deferred_lighting_without_ao_access() {
    let mut record = RenderGraphExecutionRecord::default();
    record.push_executed_pass(
        "deferred-lighting",
        "lighting.deferred",
        QueueLane::Graphics,
    );

    record.finalize_ambient_occlusion_report_for_test(
        7,
        AmbientOcclusionExecutionContract::disabled(),
    );

    let report = record.ambient_occlusion_execution_report();
    assert_eq!(
        report.status,
        RenderAmbientOcclusionExecutionStatus::Disabled
    );
    assert!(report.failure_flags.is_empty());
    assert_eq!(report.frame_generation, 7);
}

#[test]
fn complete_ao_recording_publishes_ready_generation_receipt() {
    let mut record = record_complete_ao_work();

    record.finalize_ambient_occlusion_report_for_test(7, enabled_contract());

    let report = record.ambient_occlusion_execution_report();
    assert_eq!(report.status, RenderAmbientOcclusionExecutionStatus::Ready);
    assert!(report.failure_flags.is_empty());
    assert_eq!(report.pipeline_generation, 41);
    assert_eq!(report.output_generation, 41);
    assert_eq!(report.evaluate_pass_count, 1);
    assert_eq!(report.spatial_pass_count, 1);
    assert_eq!(report.lighting_pass_count, 1);
    assert_eq!(report.evaluate_dispatch_group_count, 240);
    assert_eq!(report.spatial_dispatch_group_count, 240);
}

#[test]
fn ao_receipt_fails_when_lighting_does_not_read_final_ao() {
    let mut record = record_complete_ao_work();
    record.executed_pass_resources.last_mut().unwrap().clear();

    record.finalize_ambient_occlusion_report_for_test(7, enabled_contract());

    let report = record.ambient_occlusion_execution_report();
    assert_eq!(report.status, RenderAmbientOcclusionExecutionStatus::Failed);
    assert!(
        report
            .failure_flags
            .contains(RenderAmbientOcclusionExecutionFailureFlags::LIGHTING_FINAL_READ)
    );
}

#[test]
fn ao_receipt_fails_when_compiled_generations_diverge() {
    let mut record = record_complete_ao_work();
    let contract =
        AmbientOcclusionExecutionContract::enabled(2, 2, 3, 1, 41, 42, "ssao-spatial-denoise");

    record.finalize_ambient_occlusion_report_for_test(7, contract);

    let report = record.ambient_occlusion_execution_report();
    assert_eq!(report.status, RenderAmbientOcclusionExecutionStatus::Failed);
    assert!(
        report
            .failure_flags
            .contains(RenderAmbientOcclusionExecutionFailureFlags::GENERATION_MISMATCH)
    );
}

#[test]
fn ao_receipt_reports_compatible_last_good_pipeline_resolution() {
    let mut record = record_complete_ao_work();
    record.compute_dispatches[1].pipeline_resolution =
        Some(RenderGraphComputePipelineResolution::using_last_good(
            RenderGraphComputePipelineFamily::new("ambient-occlusion.spatial-denoise", 3),
            99,
            42,
            (7, 3),
            "candidate shader validation failed",
        ));

    record.finalize_ambient_occlusion_report_for_test(7, enabled_contract());

    let report = record.ambient_occlusion_execution_report();
    assert_eq!(
        report.status,
        RenderAmbientOcclusionExecutionStatus::UsingLastGood
    );
    assert_eq!(report.last_good_dispatch_count, 1);
    assert_eq!(report.device_id, Some(7));
    assert_eq!(report.device_generation, Some(3));
    assert_eq!(report.spatial_candidate_artifact_fingerprint, 99);
    assert_eq!(report.spatial_resolved_artifact_fingerprint, 42);
}

#[test]
fn ao_epoch_comparison_stays_typed_until_neutral_report_projection() {
    let source = include_str!("ambient_occlusion.rs");

    assert!(source.contains("use super::super::RenderPassDeviceEpoch;"));
    assert!(source.contains(") -> Option<RenderPassDeviceEpoch>"));
    assert!(source.contains("RenderPassDeviceEpoch::new("));
    assert!(source.contains("let (device_id, device_generation) = epoch.raw_parts();"));
    assert!(!source.contains("Option<(u64, u64)>"));
    assert!(!source.contains("Some(epoch.0)"));
    assert!(!source.contains("Some(epoch.1)"));
}

#[test]
fn half_resolution_ao_receipt_requires_the_bilateral_upsample_stage() {
    let mut record = record_complete_ao_work();
    let spatial_resources = &mut record.executed_pass_resources[1];
    let spatial_output = spatial_resources
        .iter_mut()
        .find(|resource| resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
        .expect("spatial output access");
    spatial_output.name = PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL.to_string();
    record.compute_dispatches[1].storage_write_resources =
        vec![PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL.to_string()];
    record.push_executed_pass_with_resources(
        "ssao-bilateral-upsample",
        "compute.generic",
        QueueLane::AsyncCompute,
        vec![
            texture_access(
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL,
                RenderGraphResourceAccessKind::Read,
            ),
            texture_access(
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
                RenderGraphResourceAccessKind::Write,
            ),
        ],
    );
    record.push_compute_dispatch(
        RenderGraphComputeDispatchRecord::new(
            "ssao-bilateral-upsample",
            "compute.generic",
            "zircon-ssao-bilateral-upsample-pipeline",
            [8, 8, 1],
            [40, 24, 1],
            vec![PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string()],
        )
        .with_pipeline_resolution(ready_pipeline_resolution(
            "ambient-occlusion.bilateral-upsample",
            43,
        )),
    );

    record.finalize_ambient_occlusion_report_for_test(
        7,
        AmbientOcclusionExecutionContract::enabled(2, 2, 3, 2, 41, 41, "ssao-bilateral-upsample"),
    );

    let report = record.ambient_occlusion_execution_report();
    assert_eq!(report.status, RenderAmbientOcclusionExecutionStatus::Ready);
    assert!(report.failure_flags.is_empty());
    assert_eq!(report.expected_pass_count, 4);
    assert_eq!(report.upsample_pass_count, 1);
    assert_eq!(report.spatial_intermediate_write_count, 1);
    assert_eq!(report.upsample_spatial_read_count, 1);
    assert_eq!(report.upsample_final_write_count, 1);
    assert_eq!(report.upsample_dispatch_group_count, 960);
}
