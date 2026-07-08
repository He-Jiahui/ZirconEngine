use super::*;

#[test]
fn runtime_15_structure_route_status_date_maps_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_PATH);
    let children = GUARD_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "structure-route map guard route owner",
        &parent,
        &[
            "#[path = \"structure_route_maps/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"structure_route_maps/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"structure_route_maps/paths.rs\"]",
            "mod paths;",
            "#[path = \"structure_route_maps/route_maps.rs\"]",
            "mod route_maps;",
            "#[path = \"structure_route_maps/row_data.rs\"]",
            "mod row_data;",
            "#[path = \"structure_route_maps/status_docs.rs\"]",
            "mod status_docs;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        ROWS_STATUS_PARENT,
        "read_structure_support_expected_slice_rows()",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "structure_route_maps.rs should delegate moved anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "structure-route map guard children",
        &children,
        &[
            "runtime_15_structure_route_status_date_maps_guard_children_stay_budgeted",
            "runtime_15_structure_route_status_date_maps_are_folder_backed",
            "runtime_15_structure_route_status_date_maps_row_data_is_synced",
            "runtime_15_structure_route_status_date_maps_docs_are_synced",
            GUARD_GUARD,
        ],
    );
}
