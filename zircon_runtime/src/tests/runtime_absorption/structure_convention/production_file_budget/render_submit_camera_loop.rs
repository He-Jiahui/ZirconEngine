use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_submit_camera_loop_tests_are_child_owner() {
    let parent = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs",
    );
    let tests = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests.rs",
    );
    let frame_tests = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests/frame.rs",
    );
    let plan_09 = read_repo("docs/plans/zircon_runtime/render/09/2026-07-09-camera-render-ordering-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let camera_loop_doc = read_repo(
        "docs/zircon_runtime/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.md",
    );
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "camera loop parent keeps production submit entry points and mounts tests",
        &parent,
        &[
            "pub(super) fn submit_camera_loop(",
            "pub(super) fn submit_camera_loop_frame(",
            "fn stream_camera_loop_extract_submissions(",
            "fn stream_camera_loop_frame_submissions(",
            "fn camera_loop_submissions(",
            "#[cfg(test)]\nmod tests;",
        ],
    );
    for moved_test in [
        "fn camera_loop_flattens_base_then_overlays_for_submit_order",
        "fn camera_loop_extracts_select_each_sequence_descriptor",
        "fn submit_camera_loop_streams_source_extract_and_restores_derived_state",
        "fn camera_loop_frame_submissions_project_selected_children_and_terminal_ui",
        "fn submit_camera_loop_frame_streams_selected_children_and_restores_source_fields",
        "fn camera_loop_extracts(",
        "fn camera_loop_frame_submissions(",
        "fn project_borrowed_frame_to_selected_camera(",
        "fn project_owned_frame_to_selected_camera(",
        "fn camera_sequence_descriptors(",
        "struct CameraLoopFrameSubmission",
        "fn descriptor(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "camera_loop.rs should mount the test child instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "camera loop test child owns focused selected-camera coverage",
        &tests,
        &[
            "use super::*;",
            "mod frame;",
            "fn camera_loop_flattens_base_then_overlays_for_submit_order",
            "fn camera_loop_extracts_select_each_sequence_descriptor",
            "fn submit_camera_loop_streams_source_extract_and_restores_derived_state",
            "fn camera_loop_extracts(",
            "fn camera_sequence_descriptors(",
            "fn descriptor(",
        ],
    );

    assert_contains_all(
        "camera loop frame test child owns direct runtime-frame projection coverage",
        &frame_tests,
        &[
            "use super::*;",
            "fn camera_loop_frame_submissions_project_selected_children_and_terminal_ui",
            "fn submit_camera_loop_frame_streams_selected_children_and_restores_source_fields",
            "fn camera_loop_frame_submissions(",
            "fn project_borrowed_frame_to_selected_camera(",
            "fn project_owned_frame_to_selected_camera(",
            "struct CameraLoopFrameSubmission",
        ],
    );

    for (path, source) in [
        (
            "graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs",
            parent.as_str(),
        ),
        (
            "graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests.rs",
            tests.as_str(),
        ),
        (
            "graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests/frame.rs",
            frame_tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production/test soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 09", plan_09.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("camera loop doc", camera_loop_doc.as_str()),
        ("render product submit doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 09 camera-loop test owner split",
                "render_plan09_camera_loop_test_owner_split_static_passed_cargo_deferred_active_editor_lane",
                "render_plan09_camera_loop_test_helper_owner_split_static_passed_cargo_deferred_active_editor_lane",
                "graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs",
                "graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests.rs",
                "graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests/frame.rs",
                "runtime_15_render_submit_camera_loop_tests_are_child_owner",
            ],
        );
    }
}
