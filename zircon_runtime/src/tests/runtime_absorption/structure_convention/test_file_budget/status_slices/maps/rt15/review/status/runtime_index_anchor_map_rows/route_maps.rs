use super::*;

#[test]
fn runtime_15_status_support_runtime_index_anchor_expected_slice_maps_are_folder_backed() {
    let status_parent = read_runtime_src(ROWS_STATUS_PARENT);
    let date_parent = read_runtime_src(ROWS_DATE_PARENT);
    let status_children = read_runtime_sources(ROWS_STATUS_CHILDREN);
    let date_children = read_runtime_sources(ROWS_DATE_CHILDREN);

    for (label, parent, function_name) in [
        (
            "status runtime-index anchor expected-slice map parent",
            status_parent.as_str(),
            "expected_status_for_slice",
        ),
        (
            "date runtime-index anchor expected-slice map parent",
            date_parent.as_str(),
            "expected_date_for_slice",
        ),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"runtime_index_anchor_maps/index_baseline_maps.rs\"]",
                "mod index_baseline_maps;",
                "#[path = \"runtime_index_anchor_maps/runtime_status_anchor_maps.rs\"]",
                "mod runtime_status_anchor_maps;",
                "#[path = \"runtime_index_anchor_maps/cargo_attempt_maps.rs\"]",
                "mod cargo_attempt_maps;",
                "#[path = \"runtime_index_anchor_maps/plan_status_guard_maps.rs\"]",
                "mod plan_status_guard_maps;",
                "#[path = \"runtime_index_anchor_maps/support_inventory_maps.rs\"]",
                "mod support_inventory_maps;",
                "#[path = \"runtime_index_anchor_maps/status_support_map_rows.rs\"]",
                "mod status_support_map_rows;",
                &format!("index_baseline_maps::{function_name}(slice)"),
                &format!("runtime_status_anchor_maps::{function_name}(slice)"),
                &format!("cargo_attempt_maps::{function_name}(slice)"),
                &format!("plan_status_guard_maps::{function_name}(slice)"),
                &format!("support_inventory_maps::{function_name}(slice)"),
                &format!("status_support_map_rows::{function_name}(slice)"),
            ],
        );
        for moved in [
            "Runtime 15 M3 runtime index subplan map 01-15 sync",
            "Runtime 15 M3 Runtime 03 module-doc status index anchor sync",
            "Runtime 15 M3 Runtime Cargo attempt status anchor sync",
            "Runtime 15 M3 plan-status index-tables child-owner split",
            "Runtime 15 M3 plan-status support inventory review sync",
            "Runtime 15 M3 status-support runtime-index anchor row-data child split",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate moved runtime-index row {moved}"
            );
        }
    }

    assert_contains_all(
        "runtime-index anchor expected-slice status/date children",
        &format!("{status_children}\n{date_children}"),
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "Some(\"2026-07-07\")",
            "runtime_15_runtime_index_subplan_map_01_15_sync_static_passed_cargo_deferred",
            "runtime_15_runtime_03_module_doc_status_index_anchor_sync_static_passed_cargo_deferred",
            "runtime_15_runtime_cargo_attempt_status_anchor_sync_static_passed_cargo_deferred",
            "runtime_15_plan_status_index_tables_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_plan_status_support_inventory_review_sync_static_passed_cargo_deferred",
            "runtime_15_status_support_runtime_index_anchor_row_data_child_split_static_passed_cargo_deferred",
        ],
    );
}
