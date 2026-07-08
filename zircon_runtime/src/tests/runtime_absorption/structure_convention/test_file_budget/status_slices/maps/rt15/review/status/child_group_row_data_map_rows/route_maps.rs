use super::*;

#[test]
fn runtime_15_status_support_child_group_row_data_maps_are_folder_backed() {
    let status_parent = read_runtime_src(ROWS_STATUS_PARENT);
    let date_parent = read_runtime_src(ROWS_DATE_PARENT);
    let status_children = read_runtime_sources(STATUS_SUPPORT_CHILD_GROUP_ROW_DATA_ROUTE_CHILDREN);
    let date_children = read_runtime_sources(DATE_SUPPORT_CHILD_GROUP_ROW_DATA_ROUTE_CHILDREN);

    for (label, parent, function_name) in [
        (
            "status child-group row-data map parent",
            status_parent.as_str(),
            "expected_status_for_slice",
        ),
        (
            "date child-group row-data map parent",
            date_parent.as_str(),
            "expected_date_for_slice",
        ),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"child_group_row_data_maps/status_doc_maps.rs\"]",
                "mod status_doc_maps;",
                "#[path = \"child_group_row_data_maps/row_data_maps.rs\"]",
                "mod row_data_maps;",
                "#[path = \"child_group_row_data_maps/status_row_doc_maps.rs\"]",
                "mod status_row_doc_maps;",
                "#[path = \"child_group_row_data_maps/moved_row_maps.rs\"]",
                "mod moved_row_maps;",
                "#[path = \"child_group_row_data_maps/expected_slice_map_rows.rs\"]",
                "mod expected_slice_map_rows;",
                &format!("status_doc_maps::{function_name}(slice)"),
                &format!("row_data_maps::{function_name}(slice)"),
                &format!("status_row_doc_maps::{function_name}(slice)"),
                &format!("moved_row_maps::{function_name}(slice)"),
                &format!("expected_slice_map_rows::{function_name}(slice)"),
            ],
        );
        for moved in [
            "Runtime 15 M3 child-groups status-doc guard child-owner split",
            "Runtime 15 M3 child-groups row-data guard folder-backed split",
            "Runtime 15 M3 child-group status-row-doc guard child-owner split",
            "Runtime 15 M3 child-group moved-row guard child-owner split",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate moved child-group row {moved}"
            );
        }
    }

    assert_contains_all(
        "child-group row-data status/date children",
        &format!("{status_children}\n{date_children}"),
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "Some(\"2026-07-07\")",
            "runtime_15_m3_child_groups_status_docs_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_m3_child_groups_row_data_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_m3_child_group_status_row_docs_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_m3_child_group_moved_row_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );
}
