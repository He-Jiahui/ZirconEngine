use super::*;

pub(in super::super) const REVIEW_GUARD_ROW_DATA_DELEGATION_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "route_mounts",
        DELEGATION_ROUTE_MOUNTS_CHILD_PATH,
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "status_inventory",
        DELEGATION_STATUS_INVENTORY_CHILD_PATH,
        "runtime_15_review_guard_row_data_delegation_status_inventory_is_current",
    ),
    (
        "child_inventory",
        DELEGATION_CHILD_INVENTORY_CHILD_PATH,
        "runtime_15_review_guard_row_data_delegation_child_inventory_is_current",
    ),
    (
        "split_layout",
        DELEGATION_SPLIT_LAYOUT_CHILD_PATH,
        DELEGATION_GUARD_FOLDER_BACKED_GUARD_NAME,
    ),
];
