use super::*;

const REVIEW_GUARD_ROW_DATA_ROOT_PATHS_CHILD_ROOT: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_paths";
const REVIEW_GUARD_ROW_DATA_ROOT_PATHS_CHILDREN: &[(&str, &str)] = &[
    ("delegation", "DELEGATION_GUARD_PATH"),
    ("foundation", "STATUS_OUTPUT_ROW_DATA_PARENT_PATH"),
    ("root_child_rows", "ROOT_CHILD_ROWS_PATH"),
    ("status_outputs", "MOVED_ROWS_GUARD_PATH"),
    ("status_support_rows", "STATUS_SUPPORT_ROWS_GUARD_PATH"),
    ("typed_error_rows", "TYPED_ERROR_ROWS_GUARD_PATH"),
];
const REVIEW_GUARD_ROW_DATA_ROOT_PATHS_STATUS_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/base_child_owner_maps.rs";
const REVIEW_GUARD_ROW_DATA_ROOT_PATHS_DATE_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/base_child_owner_maps.rs";

fn review_guard_row_data_root_paths_child_path(module_name: &str) -> String {
    format!("{REVIEW_GUARD_ROW_DATA_ROOT_PATHS_CHILD_ROOT}/{module_name}.rs")
}

#[test]
fn runtime_15_review_guard_row_data_root_paths_are_folder_backed() {
    let parent = read_runtime_src(ROOT_PATHS_PATH);
    let status_rows = review_guard_status_support_review_rows_source_blob();
    let status_map = read_runtime_src(REVIEW_GUARD_ROW_DATA_ROOT_PATHS_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_ROW_DATA_ROOT_PATHS_DATE_MAP_PATH);

    for (module_name, representative_anchor) in REVIEW_GUARD_ROW_DATA_ROOT_PATHS_CHILDREN {
        let module_mount = format!("#[path = \"root_paths/{module_name}.rs\"]");
        assert_contains_all(
            "review-guard row-data root paths route mounts focused path groups",
            &parent,
            &[module_mount.as_str(), &format!("mod {module_name};")],
        );
        assert_contains_all(
            "review-guard row-data root paths child owns representative path constants",
            &read_runtime_src(&review_guard_row_data_root_paths_child_path(module_name)),
            &["pub(in super::super) const", representative_anchor],
        );
    }

    assert_contains_all(
        "review-guard row-data root paths route mounts folder-backed guard",
        &parent,
        &[
            "#[path = \"root_paths/folder_backed.rs\"]",
            "mod folder_backed;",
        ],
    );

    for moved_anchor in [
        "pub(in super::super) const DELEGATION_GUARD_PATH",
        "pub(in super::super) const STATUS_SUPPORT_ROWS_GUARD_PATH",
        "pub(in super::super) const TYPED_ERROR_ROWS_GUARD_PATH",
        "pub(in super::super) const MOVED_ROWS_GUARD_PATH",
        "pub(in super::super) const REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH",
        "pub(in super::super) const STATUS_SUPPORT_STATUS_MAP_PATH",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review-guard row-data root paths route should not own moved anchor {moved_anchor}"
        );
    }

    assert_contains_all(
        "review-guard status rows record root paths folder-backed split",
        &status_rows,
        &[
            "Runtime 15 M3 review-guard row-data root paths folder-backed split",
            "runtime_15_review_guard_row_data_root_paths_folder_backed_static_passed_cargo_deferred",
            "runtime_15_review_guard_row_data_root_paths_are_folder_backed",
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "review-guard row-data status/date maps record root paths folder-backed split",
        &(status_map + &date_map),
        &[
            "Runtime 15 M3 review-guard row-data root paths folder-backed split",
            "runtime_15_review_guard_row_data_root_paths_folder_backed_static_passed_cargo_deferred",
            "2026-07-06",
        ],
    );
}
