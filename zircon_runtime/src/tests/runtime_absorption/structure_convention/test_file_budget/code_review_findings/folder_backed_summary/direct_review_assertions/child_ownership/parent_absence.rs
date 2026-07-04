use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_direct_review_parent_moved_guards_stay_in_children(parent: &str) {
    for f12_guard in [
        concat!(
            "F12 dead-code child owns production suppression ",
            "review guard"
        ),
        concat!(
            "review_f12_runtime_production_dead_code_",
            "suppression_is_globally_gated"
        ),
    ] {
        assert!(
            !parent.contains(f12_guard),
            "F12 direct assertion `{f12_guard}` should stay in {F12_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    for render_guard in [
        concat!(
            "render structure child owns F16 render_compiled_scene ",
            "review guard"
        ),
        concat!(
            "review_f16_compiled_scene_render_path_uses_",
            "split_owners"
        ),
    ] {
        assert!(
            !parent.contains(render_guard),
            "render direct assertion `{render_guard}` should stay in {RENDER_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    for root_parent_guard in [
        concat!(
            "code review findings parent mounts ",
            "folder-backed children"
        ),
        concat!("review_f5_world_spawn_bundle_surface_uses_", "scene_error"),
        concat!("review_d13_importer_runtime_manifests_use_", "sdk_builder"),
    ] {
        assert!(
            !parent.contains(root_parent_guard),
            "root-parent direct assertion `{root_parent_guard}` should stay in {ROOT_PARENT_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    for f8_guard in [
        concat!(
            "F8 API convergence parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "review_f8_runtime_plugin_descriptor_public_",
            "constructor_is_retired"
        ),
    ] {
        assert!(
            !parent.contains(f8_guard),
            "F8 direct assertion `{f8_guard}` should stay in {F8_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    for p0_guard in [
        concat!(
            "P0 robustness parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "P0 priority recommendation child owns current remaining-work ",
            "review guard"
        ),
        concat!(
            "review_priority_recommendation_",
            "tracks_current_remaining_work"
        ),
    ] {
        assert!(
            !parent.contains(p0_guard),
            "P0 direct assertion `{p0_guard}` should stay in {P0_DIRECT_ASSERTIONS_CHILD}"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_direct_assertions_child_ownership_guard_is_folder_backed() {
    let child_ownership_parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD);
    let child_blob = direct_assertion_child_ownership_child_source_blob();

    assert_direct_review_parent_moved_guards_stay_in_children(&read_runtime_src(
        DIRECT_REVIEW_ASSERTIONS_CHILD,
    ));
    entry_points::assert_direct_review_child_entry_points_are_current();
    budgets::assert_direct_assertions_child_ownership_children_line_budgets_are_current();
    for (_, child_path, child_guard) in DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_GUARD_CHILDREN {
        assert!(
            child_ownership_parent.contains(child_path),
            "direct assertion child-ownership parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "direct assertion child-ownership child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !child_ownership_parent.contains("#[test]"),
        "child_ownership.rs should delegate test bodies to focused children"
    );
    assert_contains_all(
        "direct assertion child-ownership parent records folder-backed status",
        &child_ownership_parent,
        &[
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_SLICE,
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_STATUS,
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_GUARD,
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_STATUS_GUARD,
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_BUDGET_GUARD,
        ],
    );
}
