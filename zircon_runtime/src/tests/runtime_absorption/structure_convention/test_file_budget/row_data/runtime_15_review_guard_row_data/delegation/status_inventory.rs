use super::*;

#[test]
fn runtime_15_review_guard_row_data_delegation_status_inventory_is_current() {
    let status_inventory = read_runtime_src(ROOT_STATUSES_PATH);

    assert_contains_all(
        "review-guard row-data status inventory records split anchors",
        &status_inventory,
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
            DELEGATION_GUARD_FOLDER_BACKED_STATUS_NAME,
            DELEGATION_GUARD_FOLDER_BACKED_STATUS_ID,
            DELEGATION_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );
}
