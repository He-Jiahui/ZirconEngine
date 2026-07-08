use super::*;

#[test]
fn runtime_15_status_support_priority_plan_doc_maps_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_PATH);
    let children = GUARD_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "priority plan-doc map guard route owner",
        &parent,
        &[
            "#[path = \"priority_plan_doc_map_rows/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"priority_plan_doc_map_rows/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"priority_plan_doc_map_rows/paths.rs\"]",
            "mod paths;",
            "#[path = \"priority_plan_doc_map_rows/route_maps.rs\"]",
            "mod route_maps;",
            "#[path = \"priority_plan_doc_map_rows/row_data.rs\"]",
            "mod row_data;",
            "#[path = \"priority_plan_doc_map_rows/status_docs.rs\"]",
            "mod status_docs;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        ROWS_STATUS_PARENT,
        "read_status_support_expected_slice_rows()",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "priority_plan_doc_map_rows.rs should delegate moved anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "priority plan-doc map guard children",
        &children,
        &[
            "runtime_15_status_support_priority_plan_doc_maps_guard_children_stay_budgeted",
            "runtime_15_status_support_priority_plan_doc_maps_are_folder_backed",
            "runtime_15_status_support_priority_plan_doc_maps_row_data_is_synced",
            "runtime_15_status_support_priority_plan_doc_maps_docs_are_synced",
            GUARD_GUARD,
        ],
    );
}
