//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。通过源码文本核对父子路由、状态镜像和文件预算。
#[path = "split_layout/route.rs"]
mod route;
#[path = "split_layout/source_inventory.rs"]
mod source_inventory;
#[path = "split_layout/sources.rs"]
mod sources;
#[path = "split_layout/status_docs.rs"]
mod status_docs;

#[test]
fn runtime_15_runtime_07_submit_context_guard_child_owner_split() {
    run_submit_context_split_layout_checks();
}

#[test]
fn runtime_15_runtime_07_submit_context_split_layout_guard_folder_backed_split() {
    run_submit_context_split_layout_checks();
}

fn run_submit_context_split_layout_checks() {
    let sources = sources::SplitLayoutSources::load();
    route::assert_submit_context_split_layout(&sources);
    source_inventory::assert_submit_context_source_inventory(&sources);
    status_docs::assert_submit_context_split_docs(&sources);
}
