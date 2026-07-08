use super::*;

#[test]
fn runtime_15_review_guard_row_data_delegation_child_inventory_is_current() {
    let child_inventory = review_guard_row_data_root_child_rows_source_blob();
    let child_sources = review_guard_row_data_child_source_blob();

    for (_, child_path, guard_name) in REVIEW_GUARD_ROW_DATA_CHILDREN {
        assert!(
            child_inventory.contains(child_path),
            "review-guard row-data child inventory should list child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "review-guard row-data child {child_path} should define {guard_name}"
        );
    }
}
