use super::*;

#[path = "mirrors/child_split_status.rs"]
mod child_split_status;
#[path = "mirrors/folder_backed_status.rs"]
mod folder_backed_status;
#[path = "mirrors/historical_status.rs"]
mod historical_status;

const EXPECTED_STATUS_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/module_layout_maps.rs";
const EXPECTED_DATE_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/module_layout_maps.rs";
const PRODUCTION_GUARD_SUPPORT_ROWS_ANCHOR: &str = "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";

const STATUS_MIRRORS_ROUTE_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summary_status/status_mirrors.rs";
const CHILD_SPLIT_STATUS_MIRROR_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summary_status/mirrors/child_split_status.rs";
const HISTORICAL_STATUS_MIRROR_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summary_status/mirrors/historical_status.rs";
const FOLDER_BACKED_STATUS_MIRROR_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summary_status/mirrors/folder_backed_status.rs";

const STATUS_MIRROR_CHILD_SPLIT_STATUS_NAME: &str =
    "Runtime 15 M3 module-layout child-summary status-doc status-mirror child split";
const STATUS_MIRROR_CHILD_SPLIT_STATUS_ID: &str = "runtime_15_module_layout_child_summary_status_docs_status_mirror_child_split_static_passed_cargo_deferred";
const STATUS_MIRROR_CHILD_SPLIT_GUARD_NAME: &str =
    "runtime_15_module_layout_child_summary_status_doc_status_mirror_children_are_folder_backed";

const STATUS_MIRROR_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_split_status",
        CHILD_SPLIT_STATUS_MIRROR_PATH,
        "runtime_15_module_layout_child_summary_status_doc_status_mirror_status_rows_are_current",
    ),
    (
        "historical_status",
        HISTORICAL_STATUS_MIRROR_PATH,
        "runtime_15_module_layout_child_summary_status_docs_are_child_owner",
    ),
    (
        "folder_backed_status",
        FOLDER_BACKED_STATUS_MIRROR_PATH,
        "runtime_15_module_layout_child_summary_status_docs_folder_backed_status_mirrors_are_current",
    ),
];

#[test]
fn runtime_15_module_layout_child_summary_status_doc_status_mirror_children_are_folder_backed() {
    let route_source = read_runtime_src(STATUS_MIRRORS_ROUTE_PATH);
    for (module_name, path, guard_name) in STATUS_MIRROR_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "module-layout child-summary status mirror route mounts child",
            &route_source,
            &[module_mount.as_str(), *path, *guard_name],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*guard_name]);
    }

    for forbidden in [
        ["let folder_backed", "_status_anchors = ["].concat(),
        ["let production_", "guard_support ="].concat(),
    ] {
        assert!(
            !route_source.contains(&forbidden),
            "module_layout_child_summary_status/status_mirrors.rs should delegate status mirror groups"
        );
    }
}
