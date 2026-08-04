use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner() {
    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let child = read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_CHILD);
    let child_blob = root_parent_direct_assertion_child_source_blob();
    let sources = super::super::super::source_inventory::code_review_findings_sources();

    assert_contains_all(
        "direct-review assertion child delegates root parent checks to child owner",
        &parent,
        &[
            "#[path = \"direct_review_assertions/root_parent.rs\"]",
            "mod root_parent;",
            "root_parent::assert_code_review_root_parent_is_folder_backed",
        ],
    );
    for moved_guard in [
        concat!(
            "code review findings parent mounts ",
            "folder-backed children"
        ),
        "code_review_findings.rs should only mount child test owners",
        concat!("review_f5_world_spawn_bundle_surface_uses_", "scene_error"),
        concat!("review_d13_importer_runtime_manifests_use_", "sdk_builder"),
        concat!(
            "review_f19_scene_renderer_construction_modules_",
            "use_construct_names"
        ),
    ] {
        assert!(
            !parent.contains(moved_guard),
            "root-parent direct assertion `{moved_guard}` should stay in {ROOT_PARENT_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    assert_contains_all(
        "root-parent direct assertion parent owns child inventory",
        &child,
        &[
            "pub(super) fn assert_code_review_root_parent_is_folder_backed",
            ROOT_PARENT_DIRECT_ASSERTIONS_DELEGATION_CHILD,
            ROOT_PARENT_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD,
            ROOT_PARENT_DIRECT_ASSERTIONS_BACKFLOW_CHILD,
            ROOT_PARENT_DIRECT_ASSERTIONS_BUDGETS_CHILD,
            "runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner",
            ROOT_PARENT_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
        ],
    );
    assert_contains_all(
        "root-parent direct assertion children own parent mount and backflow checks",
        &child_blob,
        &[
            "assert_code_review_root_parent_mounts_are_folder_backed",
            "code review findings parent mounts folder-backed children",
            "code_review_findings.rs should only mount child test owners",
            "review_f5_world_spawn_bundle_surface_uses_scene_error",
            "review_d13_importer_runtime_manifests_use_sdk_builder",
            "review_f19_scene_renderer_construction_modules_use_construct_names",
        ],
    );

    assert_code_review_root_parent_is_folder_backed(&sources);
    for (_, child_path, child_guard) in ROOT_PARENT_DIRECT_ASSERTIONS_GUARD_CHILDREN {
        assert!(
            child.contains(child_path),
            "root-parent direct assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "root-parent direct assertions child source blob should contain child guard {child_guard}"
        );
    }
    budgets::assert_root_parent_direct_assertions_children_line_budgets_are_current();
}
