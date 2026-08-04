use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_p0_robustness_parent_mounts_child_owners(sources: &CodeReviewFindingsSources) {
    assert_contains_all(
        "P0 robustness parent only mounts focused child review guard owners",
        &sources.p0_robustness,
        &[
            "#[path = \"p0_robustness/native_host_callbacks.rs\"]",
            "mod native_host_callbacks;",
            "#[path = \"p0_robustness/lock_poison.rs\"]",
            "mod lock_poison;",
            "#[path = \"p0_robustness/render_submit.rs\"]",
            "mod render_submit;",
            "#[path = \"p0_robustness/native_fixture.rs\"]",
            "mod native_fixture;",
            "#[path = \"p0_robustness/priority_recommendation.rs\"]",
            "mod priority_recommendation;",
        ],
    );
    assert_eq!(
        sources.p0_robustness.matches("#[test]").count(),
        0,
        "p0_robustness.rs should only mount child review guard owners"
    );
    for moved_test in [
        "fn review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
        "fn review_f2_scene_eventbus_locks_recover_after_poison",
        "fn review_f4_render_submit_capability_gaps_return_typed_errors",
        "fn review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
        "fn review_d13_native_fixture_importer_is_manifest_described",
        "fn review_priority_recommendation_tracks_current_remaining_work",
    ] {
        assert!(
            !sources.p0_robustness.contains(moved_test),
            "P0 robustness parent should not keep child-owned review guard `{moved_test}`"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_p0_direct_assertions_guard_is_folder_backed() {
    let p0_parent = read_runtime_src(P0_DIRECT_ASSERTIONS_CHILD);
    let child_blob = p0_direct_assertion_child_source_blob();
    let sources = super::super::super::source_inventory::code_review_findings_sources();

    assert_p0_robustness_parent_mounts_child_owners(&sources);
    review_children::assert_p0_review_children_are_folder_backed(&sources);
    budgets::assert_p0_direct_assertions_children_line_budgets_are_current();
    for (_, child_path, child_guard) in P0_DIRECT_ASSERTIONS_GUARD_CHILDREN {
        assert!(
            p0_parent.contains(child_path),
            "P0 direct assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "P0 direct assertions child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !p0_parent.contains("P0 robustness parent only mounts focused child review guard owners"),
        "p0.rs should delegate P0 parent mount assertions to parent_mounts.rs"
    );
    assert_contains_all(
        "P0 direct assertions parent records folder-backed status",
        &p0_parent,
        &[
            P0_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE,
            P0_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS,
            P0_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
            P0_DIRECT_ASSERTIONS_BUDGET_GUARD,
        ],
    );
}
