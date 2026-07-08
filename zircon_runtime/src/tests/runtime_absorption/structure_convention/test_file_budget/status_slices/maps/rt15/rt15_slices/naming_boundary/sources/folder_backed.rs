use super::*;

#[test]
fn runtime_15_status_output_naming_boundary_expected_slice_sources_are_folder_backed() {
    let parent = read_runtime_src(&format!("tests/runtime_absorption/{SOURCES_PARENT_PATH}"));
    let children = read_runtime_absorption_sources(SOURCES_CHILDREN);

    assert_contains_all(
        "naming-boundary sources route",
        &parent,
        &[
            "#[path = \"sources/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"sources/constants.rs\"]",
            "mod constants;",
            "#[path = \"sources/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"sources/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"sources/render_graphics.rs\"]",
            "mod render_graphics;",
            "#[path = \"sources/row_sources.rs\"]",
            "mod row_sources;",
            "#[path = \"sources/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"sources/structure_route_maps.rs\"]",
            "mod structure_route_maps;",
            "pub(super) use constants::*;",
            "pub(super) use guard_body::*;",
            "pub(super) use row_sources::*;",
        ],
    );
    for moved_anchor in [
        "pub(super) const SLICE",
        "pub(super) const GUARD_BODY_ROUTE_SLICE",
        "pub(super) const STATUS_PARENT_PATH",
        "pub(super) fn read_runtime_sources",
        "pub(super) fn read_status_support_expected_slice_rows",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "naming_boundary/sources.rs should delegate moved source anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "naming-boundary source children",
        &children,
        &[
            SOURCES_SLICE,
            SOURCES_STATUS,
            SOURCES_GUARD,
            SLICE,
            STATUS,
            GUARD,
            GUARD_BODY_ROUTE_SLICE,
            GUARD_BODY_ROUTE_STATUS,
            "pub(in super::super) fn read_status_support_expected_slice_rows",
            "runtime_15_naming_boundary_expected_slice_sources_children_stay_budgeted",
            "runtime_15_naming_boundary_expected_slice_sources_status_mirrors_are_synced",
        ],
    );
}
