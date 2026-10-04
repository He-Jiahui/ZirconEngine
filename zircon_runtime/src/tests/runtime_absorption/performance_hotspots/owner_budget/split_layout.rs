//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。通过源码文本核对父子路由、状态镜像和文件预算。
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
