use super::*;

#[test]
fn runtime_15_status_support_m3_m4_expected_slice_maps_are_folder_backed() {
    let status_parent = read_runtime_src(ROWS_STATUS_PARENT);
    let date_parent = read_runtime_src(ROWS_DATE_PARENT);
    let status_children = read_runtime_sources(ROWS_STATUS_CHILDREN);
    let date_children = read_runtime_sources(ROWS_DATE_CHILDREN);

    for (label, parent, function_name) in [
        (
            "status M3/M4 expected-slice map parent",
            status_parent.as_str(),
            "expected_status_for_slice",
        ),
        (
            "date M3/M4 expected-slice map parent",
            date_parent.as_str(),
            "expected_date_for_slice",
        ),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"m3_m4_expected_slice_maps/m4_row_data_maps.rs\"]",
                "mod m4_row_data_maps;",
                "#[path = \"m3_m4_expected_slice_maps/expected_slice_guard_maps.rs\"]",
                "mod expected_slice_guard_maps;",
                "#[path = \"m3_m4_expected_slice_maps/status_support_guard_maps.rs\"]",
                "mod status_support_guard_maps;",
                "#[path = \"m3_m4_expected_slice_maps/m3_row_data_maps.rs\"]",
                "mod m3_row_data_maps;",
                &format!("m4_row_data_maps::{function_name}(slice)"),
                &format!("expected_slice_guard_maps::{function_name}(slice)"),
                &format!("status_support_guard_maps::{function_name}(slice)"),
                &format!("m3_row_data_maps::{function_name}(slice)"),
            ],
        );
        for moved in [
            "Runtime 15 M3 status output Runtime 15 M4 row data split",
            "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split",
            "Runtime 15 M3 status-support route metadata row data folder-backed split",
            "Runtime 15 M3 status output Runtime 15 M3 row data split",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate moved row literal {moved}"
            );
        }
    }

    assert_contains_all(
        "status/date M3/M4 expected-slice children",
        &format!("{status_children}\n{date_children}"),
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "Some(\"2026-07-07\")",
            "Runtime 15 M3 status output Runtime 15 M4 row data split",
            "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split",
            "Runtime 15 M3 status-support route metadata row data folder-backed split",
            "Runtime 15 M3 status output Runtime 15 M3 row data split",
        ],
    );
}
