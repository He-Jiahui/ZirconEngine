use super::*;

#[path = "folder_backed_rows/exports.rs"]
mod exports;
#[path = "folder_backed_rows/status_maps.rs"]
mod status_maps;

#[test]
fn runtime_15_review_guard_direct_assertion_row_data_is_folder_backed() {
    let route = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership/folder_backed_rows.rs",
    );

    for (module, path, guard) in DIRECT_ASSERTION_ROW_OWNERSHIP_FOLDER_BACKED_CHILDREN {
        assert_contains_all(
            "direct-assertion folder-backed row route mounts child",
            &route,
            &[
                format!("#[path = \"folder_backed_rows/{module}.rs\"]").as_str(),
                format!("mod {module};").as_str(),
            ],
        );
        let child = read_runtime_src(path);
        assert!(
            child.contains(guard),
            "{path} should own direct-assertion folder-backed row guard {guard}"
        );
        assert!(
            child.lines().count() < 100,
            "{path} should stay focused after folder-backed row child split"
        );
    }
}
