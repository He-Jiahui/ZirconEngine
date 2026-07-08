use super::*;

fn read_runtime_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn runtime_15_runtime_07_script_expected_slice_maps_are_folder_backed() {
    let status_parent = read_runtime_src(ROWS_STATUS_PARENT);
    let date_parent = read_runtime_src(ROWS_DATE_PARENT);
    let status_children = read_runtime_sources(ROWS_STATUS_CHILDREN);
    let date_children = read_runtime_sources(ROWS_DATE_CHILDREN);

    for (label, parent, function_name) in [
        (
            "status Runtime 07/script expected-slice map parent",
            status_parent.as_str(),
            "expected_status_for_slice",
        ),
        (
            "date Runtime 07/script expected-slice map parent",
            date_parent.as_str(),
            "expected_date_for_slice",
        ),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"runtime07_script_maps/runtime07_guard_maps.rs\"]",
                "mod runtime07_guard_maps;",
                "#[path = \"runtime07_script_maps/runtime07_split_layout_maps.rs\"]",
                "mod runtime07_split_layout_maps;",
                "#[path = \"runtime07_script_maps/runtime07_owner_budget_maps.rs\"]",
                "mod runtime07_owner_budget_maps;",
                "#[path = \"runtime07_script_maps/script_vm_runtime_maps.rs\"]",
                "mod script_vm_runtime_maps;",
                "#[path = \"runtime07_script_maps/expected_slice_map_rows.rs\"]",
                "mod expected_slice_map_rows;",
                &format!("runtime07_guard_maps::{function_name}(slice)"),
                &format!("runtime07_split_layout_maps::{function_name}(slice)"),
                &format!("runtime07_owner_budget_maps::{function_name}(slice)"),
                &format!("script_vm_runtime_maps::{function_name}(slice)"),
                &format!("expected_slice_map_rows::{function_name}(slice)"),
            ],
        );
        for moved in [
            "Runtime 15 M3 Runtime 07 performance hotspot guard folder split",
            "Runtime 15 M3 Runtime 07 owner-budget split-layout guard folder-backed split",
            "Runtime 15 M3 Runtime 07 owner-budget virtual-geometry guard child-owner split",
            "Runtime 15 M3 script VM test folder split",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate moved row literal {moved}"
            );
        }
    }

    assert_contains_all(
        "Runtime 07/script expected-slice status/date children",
        &format!("{status_children}\n{date_children}"),
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "Some(\"2026-07-07\")",
            "runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result",
            "runtime_15_runtime_07_owner_budget_split_layout_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_runtime_07_owner_budget_virtual_geometry_guard_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_script_vm_tests_folder_split_static_passed_cargo_timeout_no_result",
        ],
    );
}
