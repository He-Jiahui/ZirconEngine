use super::*;

#[test]
fn runtime_15_status_output_row_data_module_layout_guard_is_folder_backed() {
    let module_layout_guard = format!(
        "{}\n{}",
        read_runtime_src(MODULE_LAYOUT_PARENT_PATH),
        read_runtime_src(ROOT_STATUSES_PATH)
    );
    let children_blob = module_layout_child_source_blob();

    assert_contains_all(
        "module-layout parent is a route/path inventory owner",
        &module_layout_guard,
        &[
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, _, guard) in MODULE_LAYOUT_CHILDREN {
        assert!(
            children_blob.contains(&format!("fn {guard}")),
            "module-layout child guard {guard} should live in a child file"
        );
        assert!(
            !module_layout_guard.contains(&format!("fn {guard}")),
            "module_layout.rs should not define child guard {guard}"
        );
    }
}
