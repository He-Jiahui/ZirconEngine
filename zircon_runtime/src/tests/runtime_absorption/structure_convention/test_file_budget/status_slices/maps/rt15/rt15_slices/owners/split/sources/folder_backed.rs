use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_sources_are_folder_backed()
{
    let parent = read_runtime_src(&format!("tests/runtime_absorption/{SOURCES_PARENT_PATH}"));
    let children = read_runtime_absorption_sources(SOURCES_CHILDREN);

    assert_contains_all(
        "child-owner split-layout sources route",
        &parent,
        &[
            "#[path = \"sources/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"sources/constants.rs\"]",
            "mod constants;",
            "#[path = \"sources/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"sources/row_sources.rs\"]",
            "mod row_sources;",
            "#[path = \"sources/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"sources/status_support_maps.rs\"]",
            "mod status_support_maps;",
            "pub(super) use constants::*;",
            "pub(super) use row_sources::*;",
            "pub(super) use status_support_maps::*;",
        ],
    );
    for moved_anchor in [
        "pub(super) const SLICE",
        "pub(super) const ROUTE_SLICE",
        "pub(super) const TOP_LEVEL_SUPPORT_ROWS_PATH",
        "pub(super) fn read_child_owner",
        "pub(super) fn read_status_support_expected_slice_rows",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "owners/split/sources.rs should delegate moved source anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "child-owner split-layout source children",
        &children,
        &[
            SOURCES_SLICE,
            SOURCES_STATUS,
            SOURCES_GUARD,
            SLICE,
            STATUS,
            ROUTE_SLICE,
            ROUTE_STATUS,
            "pub(in super::super) fn read_status_support_expected_slice_rows",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_sources_children_stay_budgeted",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_sources_status_mirrors_are_synced",
        ],
    );
}
