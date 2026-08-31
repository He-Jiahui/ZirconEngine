use crate::core::framework::render::{RenderAmbientOcclusionExecutionStatus, RenderStats};

use super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let report = stats.last_ambient_occlusion_execution_report;
    let tags = &["render", "ambient_occlusion", "execution"];

    record_count(
        store,
        "render.ambient_occlusion.execution.status_code",
        frame_index,
        report.status.code() as usize,
        tags,
    );
    record_bool(
        store,
        "render.ambient_occlusion.execution.ready",
        frame_index,
        report.status == RenderAmbientOcclusionExecutionStatus::Ready,
        tags,
    );
    record_bool(
        store,
        "render.ambient_occlusion.execution.using_last_good",
        frame_index,
        report.status == RenderAmbientOcclusionExecutionStatus::UsingLastGood,
        tags,
    );
    record_count(
        store,
        "render.ambient_occlusion.execution.failure_flags",
        frame_index,
        report.failure_flags.bits() as usize,
        tags,
    );
    for (path, value) in [
        (
            "render.ambient_occlusion.execution.frame_generation",
            report.frame_generation,
        ),
        (
            "render.ambient_occlusion.execution.pipeline_generation",
            report.pipeline_generation,
        ),
        (
            "render.ambient_occlusion.execution.output_generation",
            report.output_generation,
        ),
        (
            "render.ambient_occlusion.execution.device_id",
            report.device_id.unwrap_or(0),
        ),
        (
            "render.ambient_occlusion.execution.device_generation",
            report.device_generation.unwrap_or(0),
        ),
        (
            "render.ambient_occlusion.execution.evaluate_candidate_artifact_fingerprint",
            report.evaluate_candidate_artifact_fingerprint,
        ),
        (
            "render.ambient_occlusion.execution.evaluate_resolved_artifact_fingerprint",
            report.evaluate_resolved_artifact_fingerprint,
        ),
        (
            "render.ambient_occlusion.execution.spatial_candidate_artifact_fingerprint",
            report.spatial_candidate_artifact_fingerprint,
        ),
        (
            "render.ambient_occlusion.execution.spatial_resolved_artifact_fingerprint",
            report.spatial_resolved_artifact_fingerprint,
        ),
        (
            "render.ambient_occlusion.execution.upsample_candidate_artifact_fingerprint",
            report.upsample_candidate_artifact_fingerprint,
        ),
        (
            "render.ambient_occlusion.execution.upsample_resolved_artifact_fingerprint",
            report.upsample_resolved_artifact_fingerprint,
        ),
    ] {
        record_count(
            store,
            path,
            frame_index,
            usize::try_from(value).unwrap_or(usize::MAX),
            tags,
        );
    }
    for (path, value) in [
        (
            "render.ambient_occlusion.execution.profile_artifact_version",
            report.profile_artifact_version as usize,
        ),
        (
            "render.ambient_occlusion.execution.profile_compiler_version",
            report.profile_compiler_version as usize,
        ),
        (
            "render.ambient_occlusion.execution.shader_interface_version",
            report.shader_interface_version as usize,
        ),
        (
            "render.ambient_occlusion.execution.expected_pass_count",
            report.expected_pass_count,
        ),
        (
            "render.ambient_occlusion.execution.recorded_pass_count",
            report.recorded_pass_count,
        ),
        (
            "render.ambient_occlusion.execution.evaluate_pass_count",
            report.evaluate_pass_count,
        ),
        (
            "render.ambient_occlusion.execution.spatial_pass_count",
            report.spatial_pass_count,
        ),
        (
            "render.ambient_occlusion.execution.upsample_pass_count",
            report.upsample_pass_count,
        ),
        (
            "render.ambient_occlusion.execution.lighting_pass_count",
            report.lighting_pass_count,
        ),
        (
            "render.ambient_occlusion.execution.evaluate_dispatch_count",
            report.evaluate_dispatch_count,
        ),
        (
            "render.ambient_occlusion.execution.spatial_dispatch_count",
            report.spatial_dispatch_count,
        ),
        (
            "render.ambient_occlusion.execution.upsample_dispatch_count",
            report.upsample_dispatch_count,
        ),
        (
            "render.ambient_occlusion.execution.evaluate_dispatch_group_count",
            report.evaluate_dispatch_group_count,
        ),
        (
            "render.ambient_occlusion.execution.spatial_dispatch_group_count",
            report.spatial_dispatch_group_count,
        ),
        (
            "render.ambient_occlusion.execution.upsample_dispatch_group_count",
            report.upsample_dispatch_group_count,
        ),
        (
            "render.ambient_occlusion.execution.evaluate_raw_write_count",
            report.evaluate_raw_write_count,
        ),
        (
            "render.ambient_occlusion.execution.spatial_raw_read_count",
            report.spatial_raw_read_count,
        ),
        (
            "render.ambient_occlusion.execution.spatial_final_write_count",
            report.spatial_final_write_count,
        ),
        (
            "render.ambient_occlusion.execution.spatial_intermediate_write_count",
            report.spatial_intermediate_write_count,
        ),
        (
            "render.ambient_occlusion.execution.upsample_spatial_read_count",
            report.upsample_spatial_read_count,
        ),
        (
            "render.ambient_occlusion.execution.upsample_final_write_count",
            report.upsample_final_write_count,
        ),
        (
            "render.ambient_occlusion.execution.lighting_final_read_count",
            report.lighting_final_read_count,
        ),
        (
            "render.ambient_occlusion.execution.last_good_dispatch_count",
            report.last_good_dispatch_count,
        ),
    ] {
        record_count(store, path, frame_index, value, tags);
    }
}
