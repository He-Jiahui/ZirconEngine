use super::sources::{assert_contains_all, SplitLayoutSources};

const SLICE: &str =
    "Runtime 15 M3 Runtime 07 artifact/render diagnostics split-layout guard folder-backed split";
const STATUS: &str =
    "runtime_15_runtime_07_artifact_render_diagnostics_split_layout_guard_folder_backed_static_passed_cargo_deferred";
const GUARD: &str =
    "runtime_15_runtime_07_artifact_render_diagnostics_split_layout_guard_folder_backed_split";
const LEGACY_SLICE: &str =
    "Runtime 15 M3 Runtime 07 artifact/render diagnostics guard child-owner split";
const LEGACY_STATUS: &str =
    "runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split_static_passed_cargo_deferred";
const LEGACY_GUARD: &str =
    "runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split";

pub(super) fn assert_artifact_render_diagnostics_split_docs(sources: &SplitLayoutSources) {
    for (label, source) in [
        ("Runtime 15 plan", sources.runtime_15_plan),
        ("Runtime index", sources.runtime_index),
        ("review findings", sources.review_findings),
        ("structure convention", sources.structure_convention),
        ("module convention doc", sources.module_doc),
        ("Runtime 07 plan", sources.runtime_07_plan),
        ("hotspot inventory doc", sources.hotspot_doc),
        ("status-output row data", sources.status_rows),
        ("session note", sources.session_note),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                "artifact_render_diagnostics_splits/split_layout",
                GUARD,
                "expected_test_file_count = 69",
            ],
        );
    }

    assert_contains_all(
        "status-output status slice",
        sources.status_slice,
        &[SLICE, STATUS],
    );
    assert_contains_all(
        "status-output date slice",
        sources.date_slice,
        &[SLICE, "2026-07-06"],
    );

    for (label, source) in [
        ("Runtime 15 plan", sources.runtime_15_plan),
        ("Runtime index", sources.runtime_index),
        ("review findings", sources.review_findings),
        ("structure convention", sources.structure_convention),
        ("module convention doc", sources.module_doc),
        ("Runtime 07 plan", sources.runtime_07_plan),
        ("hotspot inventory doc", sources.hotspot_doc),
        ("status-output row data", sources.status_rows),
        ("session note", sources.session_note),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                LEGACY_SLICE,
                LEGACY_STATUS,
                "artifact_render_diagnostics_splits/artifact_cache_payload.rs",
                LEGACY_GUARD,
            ],
        );
    }
}
