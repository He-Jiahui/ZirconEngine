use super::*;

#[test]
fn runtime_15_status_support_expected_slice_guard_body_is_folder_backed() {
    let parent = read_runtime_src(&format!("tests/runtime_absorption/{GUARD_BODY_ROUTE_PATH}"));
    let guard_body_children = read_runtime_absorption_sources(GUARD_BODY_CHILDREN);
    let status_mirror_children = read_runtime_absorption_sources(GUARD_BODY_STATUS_MIRROR_CHILDREN);
    let children = format!("{guard_body_children}\n{status_mirror_children}");

    assert_contains_all(
        "status-support guard body route owner",
        &parent,
        &[
            "#[path = \"body/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"body/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"body/literal_ownership.rs\"]",
            "mod literal_ownership;",
            "#[path = \"body/paths.rs\"]",
            "mod paths;",
            "#[path = \"body/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"body/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "STATUS_SUPPORT_ROW_DATA_CHILD",
        "Runtime 15 M3 status output M3 row data child-owner split",
        "runtime_15_status_support_expected_slice_map_child_split_static_passed_cargo_blocked_render_environment_exports",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "status/guard_body.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "status-support guard body child tests",
        &children,
        &[
            "runtime_15_status_support_expected_slice_guard_body_children_stay_budgeted",
            GUARD_BODY_GUARD,
            "runtime_15_status_support_expected_slice_maps_are_child_owned",
            "runtime_15_status_support_expected_slice_guard_body_literals_are_child_owned",
            GUARD_BODY_STATUS_MIRRORS_GUARD,
            "runtime_15_status_support_expected_slice_status_mirrors_are_registered",
            "runtime_15_status_support_expected_slice_guard_body_status_mirrors_are_registered",
        ],
    );
}
