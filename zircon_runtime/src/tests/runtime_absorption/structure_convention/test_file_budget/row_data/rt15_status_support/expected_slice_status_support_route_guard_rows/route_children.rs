use super::*;

const STATUS_SUPPORT_ROUTE_GUARD_ROWS_PARENT: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows.rs";
const STATUS_SUPPORT_ROUTE_GUARD_ROWS_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "runtime_index_anchor_rows",
        "RUNTIME_INDEX_ANCHOR_EXPECTED_STATUS_OUTPUT_SLICES",
        "Runtime 15 M3 status-support runtime-index anchor row-data child split",
    ),
    (
        "expected_slice_route_rows",
        "EXPECTED_SLICE_ROUTE_EXPECTED_STATUS_OUTPUT_SLICES",
        "Runtime 15 M3 status-support parent-route expected-slice guard folder-backed split",
    ),
    (
        "route_input_rows",
        "ROUTE_INPUT_EXPECTED_STATUS_OUTPUT_SLICES",
        "Runtime 15 M3 status-support review-guard row-data route guard route-input folder-backed split",
    ),
    (
        "row_data_owner",
        "ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ROUTE_GUARD_ROWS_ROW_DATA_OWNER_STATUS_NAME,
    ),
];

pub(super) fn assert_route_guard_rows_are_child_owned() {
    let parent = read_runtime_src(STATUS_SUPPORT_ROUTE_GUARD_ROWS_PARENT);
    let status_support_maps = read_runtime_src(EXPECTED_SLICE_STATUS_SUPPORT_MAPS_PATH);
    for (module_name, export_name, representative_row) in STATUS_SUPPORT_ROUTE_GUARD_ROWS_CHILDREN {
        let module_mount = format!("#[path = \"route_guard_rows/{module_name}.rs\"]");
        assert_contains_all(
            "status-support route guard rows parent mounts child row groups",
            &parent,
            &[
                module_mount.as_str(),
                &format!("mod {module_name};"),
                *export_name,
            ],
        );
        assert_contains_all(
            "status-support maps route owner consumes explicit child exports",
            &status_support_maps,
            &[&format!("route_guard_rows::{export_name}")],
        );
        assert_contains_all(
            "status-support route guard row child owns representative rows",
            &read_runtime_src(&status_support_route_guard_rows_child_path(module_name)),
            &[representative_row],
        );
    }
    assert!(
        !parent.contains(
            "Runtime 15 M3 status-support parent-route expected-slice guard folder-backed split"
        ),
        "route_guard_rows.rs should route row groups instead of retaining concrete route guard rows"
    );
    assert!(
        !status_support_maps.contains("route_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES"),
        "status_support_maps.rs should consume explicit route guard row groups instead of positional indexes"
    );
    assert_contains_all(
        "status-support route guard row children retain moved rows",
        &status_support_route_guard_rows_child_source_blob(),
        &[
            "Runtime 15 M3 status-support runtime-index anchor row-data child split",
            "Runtime 15 M3 status-support parent-route expected-slice guard folder-backed split",
            "Runtime 15 M3 status-support parent-route guard route-input folder-backed split",
            ROUTE_GUARD_ROWS_ROW_DATA_OWNER_STATUS_NAME,
            ROUTE_GUARD_ROWS_ROW_DATA_OWNER_STATUS_ID,
            ROUTE_GUARD_ROWS_ROW_DATA_OWNER_GUARD_NAME,
        ],
    );
}

pub(super) fn status_support_route_guard_rows_child_source_blob() -> String {
    let mut blob = String::new();
    for (module_name, _, _) in STATUS_SUPPORT_ROUTE_GUARD_ROWS_CHILDREN {
        blob.push_str(&read_runtime_src(
            &status_support_route_guard_rows_child_path(module_name),
        ));
        blob.push('\n');
    }
    blob
}

fn status_support_route_guard_rows_child_path(module_name: &str) -> String {
    format!(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows/{module_name}.rs"
    )
}
