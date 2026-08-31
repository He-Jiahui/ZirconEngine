#[test]
fn render_stats_helpers_use_static_metadata_recording() {
    let implementation = include_str!("measurement.rs");

    assert_eq!(implementation.matches("store.record_static(").count(), 4);
    assert_eq!(implementation.matches("store.record(").count(), 0);
}

#[test]
fn render_stats_product_leaves_use_static_metadata_recording() {
    for (name, source) in [
        ("effect_stack", include_str!("product/effect_stack.rs")),
        ("light_grid", include_str!("product/light_grid.rs")),
    ] {
        assert!(
            !source.contains("store.record("),
            "{name} bypassed static render diagnostic metadata"
        );
    }
}

#[test]
fn history_domain_diagnostics_use_fixed_paths_without_frame_string_allocation() {
    let source = include_str!("history.rs");

    for domain in [
        "taa_scene_color",
        "hybrid_global_illumination",
        "ambient_occlusion",
        "screen_space_reflection",
        "hzb_furthest",
        "exposure",
        "volumetric_scattering",
    ] {
        assert!(
            source.contains(&format!("render.history.domain.{domain}.valid")),
            "missing fixed diagnostic path for {domain}"
        );
        assert!(
            source.contains(&format!(
                "render.history.domain.{domain}.active_reset_reason_code"
            )),
            "missing active reset diagnostic path for {domain}"
        );
        assert!(
            source.contains(&format!(
                "render.history.domain.{domain}.frame_reset_reason_code"
            )),
            "missing frame reset diagnostic path for {domain}"
        );
    }
    assert!(!source.contains("format!("));
}

#[test]
fn ambient_occlusion_execution_diagnostics_use_fixed_receipt_paths() {
    let source = include_str!("ambient_occlusion.rs");

    assert_eq!(
        source
            .matches("render.ambient_occlusion.execution.")
            .count(),
        38
    );

    for leaf in [
        "status_code",
        "using_last_good",
        "failure_flags",
        "pipeline_generation",
        "output_generation",
        "device_generation",
        "evaluate_candidate_artifact_fingerprint",
        "spatial_resolved_artifact_fingerprint",
        "upsample_resolved_artifact_fingerprint",
        "last_good_dispatch_count",
        "evaluate_dispatch_count",
        "spatial_dispatch_count",
        "upsample_dispatch_count",
        "spatial_intermediate_write_count",
        "upsample_final_write_count",
        "lighting_final_read_count",
    ] {
        assert!(source.contains(&format!("render.ambient_occlusion.execution.{leaf}")));
    }
    assert!(!source.contains("format!("));
}

#[test]
fn scene_submission_completion_diagnostics_use_fixed_receipt_paths() {
    let source = include_str!("scene_submission_completion.rs");

    assert_eq!(source.matches("render.submission.completion.").count(), 11);
    for leaf in [
        "status_code",
        "failure_code",
        "completed",
        "frame_generation",
        "submission_sequence",
        "poll_sequence",
        "device_generation",
        "pending_submission_count",
        "tracking_capacity",
        "last_poll_observed_submission_count",
        "last_poll_terminal_submission_count",
    ] {
        assert!(source.contains(&format!("render.submission.completion.{leaf}")));
    }
    assert!(!source.contains("format!("));
}
