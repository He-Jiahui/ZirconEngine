use super::*;

#[test]
fn runtime_15_status_support_priority_plan_doc_maps_are_folder_backed() {
    let status_parent = read_runtime_src(ROWS_STATUS_PARENT);
    let date_parent = read_runtime_src(ROWS_DATE_PARENT);
    let status_children = read_runtime_sources(STATUS_SUPPORT_PRIORITY_PLAN_DOC_ROUTE_CHILDREN);
    let date_children = read_runtime_sources(DATE_SUPPORT_PRIORITY_PLAN_DOC_ROUTE_CHILDREN);

    for (label, parent, function_name) in [
        (
            "status priority plan-doc map parent",
            status_parent.as_str(),
            "expected_status_for_slice",
        ),
        (
            "date priority plan-doc map parent",
            date_parent.as_str(),
            "expected_date_for_slice",
        ),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"priority_plan_doc_maps/integrity_guard_maps.rs\"]",
                "mod integrity_guard_maps;",
                "#[path = \"priority_plan_doc_maps/guard_child_owner_maps.rs\"]",
                "mod guard_child_owner_maps;",
                "#[path = \"priority_plan_doc_maps/inventory_sync_maps.rs\"]",
                "mod inventory_sync_maps;",
                "#[path = \"priority_plan_doc_maps/row_data_guard_maps.rs\"]",
                "mod row_data_guard_maps;",
                "#[path = \"priority_plan_doc_maps/status_mirror_maps.rs\"]",
                "mod status_mirror_maps;",
                "#[path = \"priority_plan_doc_maps/expected_slice_map_rows.rs\"]",
                "mod expected_slice_map_rows;",
                &format!("integrity_guard_maps::{function_name}(slice)"),
                &format!("guard_child_owner_maps::{function_name}(slice)"),
                &format!("inventory_sync_maps::{function_name}(slice)"),
                &format!("row_data_guard_maps::{function_name}(slice)"),
                &format!("status_mirror_maps::{function_name}(slice)"),
                &format!("expected_slice_map_rows::{function_name}(slice)"),
            ],
        );
        for moved in [
            "Runtime 15 M3 priority plan docs code-path integrity guard",
            "Runtime 15 M3 priority plan docs guard child-owner split",
            "Runtime 15 M3 priority plan docs listing prose full inventory sync",
            "Runtime 15 M3 priority plan docs row-data owner child split",
            "Runtime 15 M3 priority plan docs status-mirror child split",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate moved priority plan-doc row {moved}"
            );
        }
    }

    assert_contains_all(
        "priority plan-doc status/date children",
        &format!("{status_children}\n{date_children}"),
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "Some(\"2026-07-07\")",
            "runtime_15_priority_plan_docs_code_path_integrity_guard_static_passed_cargo_deferred",
            "runtime_15_priority_plan_docs_guard_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_priority_plan_docs_root_inventory_child_split_static_passed_cargo_deferred",
            "runtime_15_priority_plan_docs_row_data_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_priority_plan_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ],
    );
}
