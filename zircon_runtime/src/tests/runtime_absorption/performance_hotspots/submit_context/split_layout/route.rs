use super::sources::{assert_contains_all, SplitLayoutSources};

pub(super) fn assert_submit_context_split_layout(sources: &SplitLayoutSources) {
    assert_submit_context_parent_route(sources);
    assert_submit_context_support_children(sources);
    assert_submit_context_split_route(sources);
    assert_submit_context_split_budgets(sources);
}

fn assert_submit_context_parent_route(sources: &SplitLayoutSources) {
    assert_contains_all(
        "submit_context route",
        sources.parent,
        &[
            "#[path = \"submit_context/camera_loop_sharing.rs\"]",
            "#[path = \"submit_context/feedback_sidebands.rs\"]",
            "#[path = \"submit_context/source_extract_payloads.rs\"]",
            "#[path = \"submit_context/sources.rs\"]",
            "#[path = \"submit_context/split_layout.rs\"]",
            "#[path = \"submit_context/status_docs.rs\"]",
            "fn runtime_07_submit_context_shares_large_extract_payloads()",
            "source_extract_payloads::assert_source_extract_payloads_are_shared(&sources);",
            "camera_loop_sharing::assert_camera_loop_uses_shared_sources(&sources);",
            "feedback_sidebands::assert_feedback_sidebands_move_owned_payloads(&sources);",
            "status_docs::assert_submit_context_status_docs(&sources);",
        ],
    );

    for moved_anchor in [
        "for forbidden_owned_payload in [",
        "let camera_loop_submission_body = camera_loop",
        "for required_feedback_anchor in [",
        "for status_anchor in [",
    ] {
        assert!(
            !sources.parent.contains(moved_anchor),
            "submit_context.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }
}

fn assert_submit_context_support_children(sources: &SplitLayoutSources) {
    assert_contains_all(
        "submit_context sources",
        sources.submit_sources,
        &[
            "pub(super) struct SubmitContextSources",
            "pub(super) fn load() -> Self",
            "frame_submission_context.rs",
            "submit_runtime_frame.rs",
            "07-runtime-performance-hotpath.md",
        ],
    );
    assert_contains_all(
        "source extract child",
        sources.source_extract_payloads,
        &[
            "assert_source_extract_payloads_are_shared",
            "source_extract: Arc<RenderFrameExtract>",
            "FrameSubmissionSourcePayloads",
            "ViewportRenderFrame::from_shared_extract",
        ],
    );
    assert_contains_all(
        "camera loop child",
        sources.camera_loop_sharing,
        &[
            "assert_camera_loop_uses_shared_sources",
            "CameraLoopExtractSourceState",
            "CameraLoopFrameSourceState::capture(&mut frame)",
            "render_frame_with_pipeline",
        ],
    );
    assert_contains_all(
        "feedback sideband child",
        sources.feedback_sidebands,
        &[
            "assert_feedback_sidebands_move_owned_payloads",
            "sidebands.take_hybrid_gi_readback_outputs()",
            "fn into_prepared_runtime_sidebands(self) -> RenderPreparedRuntimeSidebands",
        ],
    );
    assert_contains_all(
        "status docs child",
        sources.status_docs,
        &[
            "assert_submit_context_status_docs",
            "Runtime 07 render submit source-extract sharing",
            "runtime_07_submit_context_shares_large_extract_payloads",
        ],
    );
}

fn assert_submit_context_split_route(sources: &SplitLayoutSources) {
    assert_contains_all(
        "submit-context split-layout route",
        sources.split_layout,
        &[
            "#[path = \"split_layout/route.rs\"]",
            "#[path = \"split_layout/source_inventory.rs\"]",
            "#[path = \"split_layout/sources.rs\"]",
            "#[path = \"split_layout/status_docs.rs\"]",
            "runtime_15_runtime_07_submit_context_guard_child_owner_split",
            "runtime_15_runtime_07_submit_context_split_layout_guard_folder_backed_split",
            "run_submit_context_split_layout_checks();",
            "route::assert_submit_context_split_layout(&sources);",
            "source_inventory::assert_submit_context_source_inventory(&sources);",
            "status_docs::assert_submit_context_split_docs(&sources);",
        ],
    );

    for moved_anchor in [
        "let parent = include_str!(\"../submit_context.rs\")",
        "let source_inventory = include_str!",
        "for moved_anchor in [",
        "for (path, source) in [",
        "for (label, source) in [",
    ] {
        assert!(
            !sources.split_layout.contains(moved_anchor),
            "submit_context/split_layout.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "submit-context split-layout children",
        &format!(
            "{}\n{}\n{}\n{}",
            sources.split_layout_route,
            sources.split_layout_source_inventory,
            sources.split_layout_sources,
            sources.split_layout_status_docs
        ),
        &[
            "assert_submit_context_split_layout",
            "assert_submit_context_source_inventory",
            "pub(super) struct SplitLayoutSources",
            "assert_submit_context_split_docs",
            "Runtime 15 M3 Runtime 07 submit-context split-layout guard folder-backed split",
        ],
    );
}

fn assert_submit_context_split_budgets(sources: &SplitLayoutSources) {
    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/submit_context.rs",
            sources.parent,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/sources.rs",
            sources.submit_sources,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/source_extract_payloads.rs",
            sources.source_extract_payloads,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/camera_loop_sharing.rs",
            sources.camera_loop_sharing,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/feedback_sidebands.rs",
            sources.feedback_sidebands,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/status_docs.rs",
            sources.status_docs,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/split_layout.rs",
            sources.split_layout,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/split_layout/route.rs",
            sources.split_layout_route,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/split_layout/source_inventory.rs",
            sources.split_layout_source_inventory,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/split_layout/sources.rs",
            sources.split_layout_sources,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context/split_layout/status_docs.rs",
            sources.split_layout_status_docs,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 260,
            "{path} should stay below the focused submit-context split guard budget; got {line_count} lines"
        );
    }
}
