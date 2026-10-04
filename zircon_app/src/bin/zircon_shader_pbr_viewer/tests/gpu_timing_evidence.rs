use super::*;

use crate::work_paths::viewer_test_artifact_root;

fn mesh_submission() -> RenderMeshSubmissionProfile {
    RenderMeshSubmissionProfile {
        opaque_command_count: 1,
        advanced_pbr_opaque_command_count: 0,
        cached_command_hit_count: 1,
        command_rebuild_count: 0,
        dynamic_command_count: 0,
        ..RenderMeshSubmissionProfile::default()
    }
}

fn report(generation: u64, pass_name: &str, gpu_time_us: u64) -> SceneRendererGpuTimingReport {
    SceneRendererGpuTimingReport::new(
        generation,
        1.0,
        [SceneRendererGpuPassTiming::new(pass_name, gpu_time_us)],
    )
    .with_mesh_submission_profile(mesh_submission())
}

fn direct_report(generation: u64, base_gpu_time_us: u64) -> SceneRendererGpuTimingReport {
    SceneRendererGpuTimingReport::new(
        generation,
        1.0,
        [
            SceneRendererGpuPassTiming::new("direct_gpu_scene_upload", 0),
            SceneRendererGpuPassTiming::new("direct_scene_content", base_gpu_time_us),
            SceneRendererGpuPassTiming::new("direct_output_transfer", base_gpu_time_us + 1),
            SceneRendererGpuPassTiming::new("direct_overlays", base_gpu_time_us + 2),
        ],
    )
    .with_mesh_submission_profile(mesh_submission())
}

fn measured_distribution(screenshot_generation: u64) -> GpuTimingEvidenceResolution {
    let mut request = GpuTimingEvidenceRequest::new(screenshot_generation);
    let mut resolution = GpuTimingEvidenceResolution::Pending;
    for generation in screenshot_generation + 1
        ..=screenshot_generation
            + (GPU_TIMING_WARMUP_SAMPLE_COUNT + GPU_TIMING_MEASURED_SAMPLE_COUNT) as u64
    {
        resolution = request.observe(
            Some(direct_report(generation, generation)),
            RenderGpuTimingStatus::Pending,
        );
    }
    resolution
}

#[test]
fn request_discards_the_screenshot_frame_then_collects_warmup_and_distribution() {
    let mut request = GpuTimingEvidenceRequest::new(7);

    assert_eq!(
        request.observe(Some(direct_report(7, 42)), RenderGpuTimingStatus::Pending),
        GpuTimingEvidenceResolution::Pending
    );
    for generation in 8..8 + GPU_TIMING_WARMUP_SAMPLE_COUNT as u64 {
        assert_eq!(
            request.observe(
                Some(direct_report(generation, generation)),
                RenderGpuTimingStatus::Pending,
            ),
            GpuTimingEvidenceResolution::Pending
        );
    }
    for generation in 13..43 {
        assert_eq!(
            request.observe(
                Some(direct_report(generation, generation)),
                RenderGpuTimingStatus::Pending,
            ),
            GpuTimingEvidenceResolution::Pending
        );
    }
    let resolution = request.observe(Some(direct_report(43, 43)), RenderGpuTimingStatus::Pending);
    let GpuTimingEvidenceResolution::Measured(distribution) = resolution else {
        panic!("the thirty-first stable sample must complete the distribution");
    };
    assert_eq!(distribution.screenshot_frame_generation(), 7);
    assert_eq!(distribution.first_measured_frame_generation(), 13);
    assert_eq!(distribution.last_measured_frame_generation(), 43);
    assert_eq!(
        distribution.samples().len(),
        GPU_TIMING_MEASURED_SAMPLE_COUNT
    );
}

#[test]
fn request_times_out_after_a_finite_nonblocking_resolution_budget() {
    let mut request = GpuTimingEvidenceRequest::new(7);

    for _ in 1..MAX_GPU_TIMING_RESOLVE_FRAMES {
        assert_eq!(
            request.observe(None, RenderGpuTimingStatus::Pending),
            GpuTimingEvidenceResolution::Pending
        );
    }
    assert_eq!(
        request.observe(None, RenderGpuTimingStatus::Pending),
        GpuTimingEvidenceResolution::TimedOut
    );
}

#[test]
fn request_fails_closed_on_a_generation_gap_or_pass_coverage_drift() {
    let mut generation_gap = GpuTimingEvidenceRequest::new(7);
    assert_eq!(
        generation_gap.observe(Some(direct_report(8, 8)), RenderGpuTimingStatus::Pending,),
        GpuTimingEvidenceResolution::Pending
    );
    assert!(matches!(
        generation_gap.observe(Some(direct_report(10, 10)), RenderGpuTimingStatus::Pending,),
        GpuTimingEvidenceResolution::Invalid(_)
    ));

    let mut mesh_drift = GpuTimingEvidenceRequest::new(7);
    for generation in 8..=13 {
        assert_eq!(
            mesh_drift.observe(
                Some(direct_report(generation, generation)),
                RenderGpuTimingStatus::Pending,
            ),
            GpuTimingEvidenceResolution::Pending
        );
    }
    let changed_mesh_submission = RenderMeshSubmissionProfile {
        advanced_pbr_opaque_command_count: 1,
        ..mesh_submission()
    };
    let changed_report = SceneRendererGpuTimingReport::new(
        14,
        1.0,
        [
            SceneRendererGpuPassTiming::new("direct_gpu_scene_upload", 0),
            SceneRendererGpuPassTiming::new("direct_scene_content", 14),
            SceneRendererGpuPassTiming::new("direct_output_transfer", 15),
            SceneRendererGpuPassTiming::new("direct_overlays", 16),
        ],
    )
    .with_mesh_submission_profile(changed_mesh_submission);
    assert!(matches!(
        mesh_drift.observe(Some(changed_report), RenderGpuTimingStatus::Pending),
        GpuTimingEvidenceResolution::Invalid(reason)
            if reason.contains("mesh submission changed")
    ));

    let mut coverage_drift = GpuTimingEvidenceRequest::new(7);
    assert_eq!(
        coverage_drift.observe(Some(direct_report(8, 8)), RenderGpuTimingStatus::Pending,),
        GpuTimingEvidenceResolution::Pending
    );
    assert!(matches!(
        coverage_drift.observe(
            Some(report(9, "direct_scene_content", 9)),
            RenderGpuTimingStatus::Pending,
        ),
        GpuTimingEvidenceResolution::Invalid(_)
    ));

    let mut missing_mesh_submission = GpuTimingEvidenceRequest::new(7);
    assert!(matches!(
        missing_mesh_submission.observe(
            Some(SceneRendererGpuTimingReport::new(
                8,
                1.0,
                [SceneRendererGpuPassTiming::new("direct_scene_content", 8)],
            )),
            RenderGpuTimingStatus::Pending,
        ),
        GpuTimingEvidenceResolution::Invalid(reason)
            if reason.contains("missing its matching mesh submission")
    ));
}

#[test]
fn deferred_sampling_is_terminal_instead_of_silently_dropping_a_frame() {
    let mut request = GpuTimingEvidenceRequest::new(7);

    assert_eq!(
        request.observe(None, RenderGpuTimingStatus::Deferred),
        GpuTimingEvidenceResolution::Unavailable(RenderGpuTimingStatus::Deferred)
    );
}

#[test]
fn unavailable_timestamp_support_is_reported_without_waiting_for_the_budget() {
    let mut request = GpuTimingEvidenceRequest::new(7);

    assert_eq!(
        request.observe(None, RenderGpuTimingStatus::Unavailable),
        GpuTimingEvidenceResolution::Unavailable(RenderGpuTimingStatus::Unavailable)
    );
}

#[test]
fn measured_evidence_keeps_raw_samples_calibration_and_verified_percentiles() {
    let resolution = measured_distribution(7);

    let output =
        format_gpu_timing_evidence_with_screenshot_identity("ready.png", "deadbeef", &resolution);
    assert!(output
        .starts_with("schema=zircon_shader_pbr_viewer_gpu_timing_evidence_v3\nstatus=measured\n"));
    assert!(output.contains("screenshot_frame_generation=7\n"));
    assert!(output.contains("warmup_sample_count=5\n"));
    assert!(output.contains("measured_sample_count=31\n"));
    assert!(output.contains("first_measured_frame_generation=13\n"));
    assert!(output.contains("last_measured_frame_generation=43\n"));
    assert!(output.contains("timestamp_period_ns=1.000000000\n"));
    assert!(output.contains("timestamp_frequency_hz=1000000000.000\n"));
    assert!(output.contains("percentile_policy=nearest_rank\n"));
    assert!(output.contains("outlier_policy=none_all_samples_retained\n"));
    assert!(output.contains("pass.direct_scene_content.median_us=28\n"));
    assert!(output.contains("pass.direct_scene_content.p95_us=42\n"));
    assert!(output.contains("sample.000.frame_generation=13\n"));
    assert!(output.contains("sample.030.frame_generation=43\n"));
    assert!(output.contains("sample.030.pass.direct_overlays_us=45\n"));
    assert!(output.contains("sample.000.mesh.opaque_command_count=1\n"));
    assert!(output.contains("sample.030.mesh.dynamic_command_count=0\n"));
}

#[test]
fn screenshot_identity_uses_the_standard_sha256_digest() {
    assert_eq!(
        screenshot_sha256_bytes(b"fixture"),
        "f16d05ec6b29248d2c61adb1e9263f78e4f7bace1b955014a2d17872cfe4064d"
    );
}

#[test]
fn formatter_hashes_the_actual_ready_png_file() {
    let artifact_root = viewer_test_artifact_root("gpu-timing-screenshot-hash");
    let screenshot_path = artifact_root.join("ready.png");
    std::fs::write(&screenshot_path, b"fixture")
        .expect("controlled screenshot fixture should be written");

    let output = format_gpu_timing_evidence(&screenshot_path, &measured_distribution(7))
        .expect("a written Ready PNG should produce timing evidence");
    std::fs::remove_dir_all(&artifact_root)
        .expect("controlled screenshot fixture root should be removed");

    assert!(output.contains("screenshot=ready.png\n"));
    assert!(output.contains(
        "screenshot_sha256=f16d05ec6b29248d2c61adb1e9263f78e4f7bace1b955014a2d17872cfe4064d\n"
    ));
}

#[test]
fn single_component_gpu_timing_report_path_has_no_directory_to_create() {
    assert_eq!(gpu_timing_report_parent(Path::new("timing.txt")), None);
    assert_eq!(
        gpu_timing_report_parent(Path::new("evidence/timing.txt")),
        Some(Path::new("evidence"))
    );
    assert!(
        include_str!("../app.rs").contains("if let Some(parent) = gpu_timing_report_parent(path)")
    );
}

#[test]
fn timing_output_must_not_collide_with_the_ready_png_or_its_sidecar() {
    for (screenshot_path, report_path) in [
        (
            "E:/evidence/pbr-ready.png",
            "E:/evidence/frames/../pbr-ready.png",
        ),
        ("E:/evidence/pbr-ready.png", "E:/evidence/pbr-ready.png.txt"),
        ("E:/Evidence/PBR-READY.PNG", "e:/evidence/pbr-ready.png"),
        (
            "docs/tests/runtime/shader/./pbr-ready.png",
            "./docs/tests/runtime/shader/pbr-ready.png",
        ),
    ] {
        let error =
            validate_gpu_timing_report_output(Path::new(screenshot_path), Path::new(report_path))
                .expect_err("timing evidence must not overwrite Ready evidence");
        assert!(error.contains("must not overwrite"));
    }
    validate_gpu_timing_report_output(
        Path::new("E:/evidence/pbr-ready.png"),
        Path::new("E:/evidence/pbr-gpu-timing.txt"),
    )
    .expect("a separate GPU timing report should remain valid");
}
