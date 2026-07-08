use super::*;

#[test]
fn runtime_15_status_support_expected_slice_guard_body_status_mirrors_are_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{GUARD_BODY_STATUS_MIRRORS_ROUTE_PATH}"
    ));
    let children = read_runtime_absorption_sources(GUARD_BODY_STATUS_MIRROR_CHILDREN);

    assert_contains_all(
        "status-support guard body status mirror route owner",
        &parent,
        &[
            "#[path = \"mirrors/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"mirrors/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"mirrors/guard_body_status.rs\"]",
            "mod guard_body_status;",
            "#[path = \"mirrors/map_child_split.rs\"]",
            "mod map_child_split;",
            "#[path = \"mirrors/status_docs.rs\"]",
            "mod status_docs;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "read_status_support_expected_slice_rows",
        "Runtime 15 M3 status-support expected-slice map child split",
        GUARD_BODY_STATUS_MAP_PATH,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "status/body/status_mirrors.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "status-support guard body status mirror children",
        &children,
        &[
            "runtime_15_status_support_expected_slice_guard_body_status_mirrors_children_stay_budgeted",
            GUARD_BODY_STATUS_MIRRORS_GUARD,
            "runtime_15_status_support_expected_slice_status_mirrors_are_registered",
            "runtime_15_status_support_expected_slice_guard_body_status_mirrors_are_registered",
            "runtime_15_status_support_expected_slice_guard_body_status_mirrors_status_is_synced",
        ],
    );
}
