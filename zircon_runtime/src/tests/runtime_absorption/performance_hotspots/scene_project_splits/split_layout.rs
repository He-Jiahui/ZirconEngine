#[path = "split_layout/route.rs"]
mod route;
#[path = "split_layout/source_inventory.rs"]
mod source_inventory;
#[path = "split_layout/sources.rs"]
mod sources;
#[path = "split_layout/status_docs.rs"]
mod status_docs;

#[test]
fn runtime_15_runtime_07_scene_project_guard_child_owner_split() {
    run_scene_project_split_layout_checks();
}

#[test]
fn runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_split() {
    run_scene_project_split_layout_checks();
}

fn run_scene_project_split_layout_checks() {
    let sources = sources::SplitLayoutSources::load();
    route::assert_scene_project_split_layout(&sources);
    source_inventory::assert_scene_project_source_inventory(&sources);
    status_docs::assert_scene_project_split_docs(&sources);
}
