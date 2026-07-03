use super::*;

#[test]
fn runtime_15_p0_native_fixture_leaf_owner_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_OWNER);
    let child_sources = folder_backed_child_source_blob();

    assert_contains_all(
        "P0 native fixture leaf-owner structure guard delegates focused children",
        &parent,
        &[
            "#[path = \"p0_native_fixture_leaf_owners/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"p0_native_fixture_leaf_owners/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"p0_native_fixture_leaf_owners/leaf_ownership.rs\"]",
            "mod leaf_ownership;",
            "#[path = \"p0_native_fixture_leaf_owners/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(super) const STRUCTURE_GUARD_OWNER",
            FOLDER_BACKED_SLICE,
            FOLDER_BACKED_STATUS,
        ],
    );
    for moved_guard in [
        format!("fn {GUARD}"),
        format!("fn {FOLDER_BACKED_STATUS_GUARD}"),
        format!("fn {BUDGET_GUARD}"),
    ] {
        assert!(
            !parent.contains(&moved_guard),
            "P0 native fixture leaf-owner guard `{moved_guard}` should stay in child files"
        );
    }
    for (_, child_path, guard_name) in FOLDER_BACKED_CHILDREN {
        assert!(
            parent.contains(child_path),
            "P0 native fixture leaf-owner parent should inventory child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "P0 native fixture leaf-owner child {child_path} should define {guard_name}"
        );
    }
}
