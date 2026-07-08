#[path = "split_layout/route.rs"]
mod route;
#[path = "split_layout/source_inventory.rs"]
mod source_inventory;
#[path = "split_layout/status_docs.rs"]
mod status_docs;

#[test]
fn runtime_15_runtime_07_owner_budget_guard_folder_backed_split() {
    let sources = super::sources::load();
    route::assert_owner_budget_split_layout(&sources);
    source_inventory::assert_owner_budget_source_inventory(&sources);
    source_inventory::assert_owner_budget_split_budgets(&sources);
    status_docs::assert_owner_budget_split_docs(&sources);
}

#[test]
fn runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_split() {
    let sources = super::sources::load();
    route::assert_owner_budget_split_layout_route_folder_backed(&sources);
}
