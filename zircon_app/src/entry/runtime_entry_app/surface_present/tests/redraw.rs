use std::num::NonZeroU64;

use zircon_runtime::asset::project::ProjectPaths;

use super::{
    presented_frame_exit_diagnostic, should_capture_first_presented_frame,
    should_emit_first_frame_product_diagnostics,
};

#[test]
fn first_frame_exit_emits_a_presented_frame_diagnostic() {
    assert_eq!(
        presented_frame_exit_diagnostic(1, Some(NonZeroU64::MIN)),
        Some("runtime_first_frame_presented".to_string())
    );
}

#[test]
fn presented_frame_exit_waits_for_the_configured_successful_present_count() {
    let limit = NonZeroU64::new(120).unwrap();

    assert_eq!(presented_frame_exit_diagnostic(119, Some(limit)), None);
    assert_eq!(
        presented_frame_exit_diagnostic(120, Some(limit)),
        Some("runtime_presented_frame_limit_reached limit=120 count=120".to_string())
    );
    assert_eq!(presented_frame_exit_diagnostic(120, None), None);
    assert!(should_emit_first_frame_product_diagnostics(false));
}

#[test]
fn product_frame_diagnostics_are_not_repeated_after_the_first_present() {
    assert!(!should_emit_first_frame_product_diagnostics(true));
}

#[test]
fn requested_first_frame_capture_runs_once_after_a_presented_frame() {
    let path = ProjectPaths::resolve_path("E:/evidence/runtime-first-frame.png")
        .expect("capture path should resolve");

    assert!(should_capture_first_presented_frame(Some(&path), false));
    assert!(!should_capture_first_presented_frame(Some(&path), true));
    assert!(!should_capture_first_presented_frame(None, false));
}

#[test]
fn frame_capture_projects_to_the_runtime_entry_root_sibling() {
    let source = include_str!("../redraw.rs");
    let writer = ["frame_capture::", "write_runtime_frame_png"].concat();
    let root_sibling = ["super::super::", writer.as_str()].concat();
    let expected_call = format!("        {root_sibling}(");
    let calls = source
        .lines()
        .filter(|line| line.contains(&writer))
        .collect::<Vec<_>>();

    assert_eq!(calls, vec![expected_call.as_str()]);
}

#[test]
fn present_paths_keep_the_p1_capture_and_present_measurement_points() {
    let source = include_str!("../redraw.rs");

    for name in [
        "runtime_entry.native_present",
        "runtime_entry.reference_cpu_presenter.capture_request",
        "runtime_entry.reference_cpu_presenter.presented",
        "runtime_entry.presented_frame",
        "runtime_entry.explicit_frame_capture_request",
        "runtime_entry.explicit_frame_capture_rgba_bytes",
    ] {
        assert!(
            source.contains(name),
            "P1 presentation reporting must retain the `{name}` counter"
        );
    }
}
