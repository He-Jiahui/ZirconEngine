use super::*;

#[test]
fn runtime_15_review_guard_row_data_delegation_guard_is_folder_backed() {
    let parent = read_runtime_src(DELEGATION_GUARD_PATH);
    let route_mounts = read_runtime_src(DELEGATION_ROUTE_MOUNTS_CHILD_PATH);
    let status_inventory = read_runtime_src(DELEGATION_STATUS_INVENTORY_CHILD_PATH);
    let child_inventory = read_runtime_src(DELEGATION_CHILD_INVENTORY_CHILD_PATH);
    let split_layout = read_runtime_src(DELEGATION_SPLIT_LAYOUT_CHILD_PATH);
    let split_route_mounts = read_runtime_src(DELEGATION_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH);

    assert_contains_all(
        "review-guard row-data delegation guard mounts focused children",
        &parent,
        &[
            "#[path = \"delegation/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"delegation/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"delegation/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"delegation/status_inventory.rs\"]",
            "mod status_inventory;",
        ],
    );
    for moved_anchor in [
        "let status_output_row_data_parent = read_runtime_src",
        "review-guard row-data status inventory records split anchors",
        "review-guard row-data child inventory should list child path",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "delegation.rs should stay a route owner and delegate {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-guard row-data delegation children retain moved checks",
        &format!(
            "{route_mounts}\n{status_inventory}\n{child_inventory}\n{split_layout}\n{split_route_mounts}"
        ),
        &[
            CHILD_OWNER_GUARD_NAME,
            "runtime_15_review_guard_row_data_delegation_status_inventory_is_current",
            "runtime_15_review_guard_row_data_delegation_child_inventory_is_current",
            DELEGATION_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );
    assert_contains_all(
        "review-guard row-data delegation split-layout routes focused children",
        &split_layout,
        &[
            "#[path = \"split/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"split/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"split/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"split/status_current.rs\"]",
            "mod status_current;",
        ],
    );
}
