#[path = "owner_budget/child_routes.rs"]
mod child_routes;
#[path = "owner_budget/large_file_gate.rs"]
mod large_file_gate;
#[path = "owner_budget/line_budgets.rs"]
mod line_budgets;
#[path = "owner_budget/mirror_docs.rs"]
mod mirror_docs;
#[path = "owner_budget/parent_routes.rs"]
mod parent_routes;
#[path = "owner_budget/source_inventory.rs"]
mod source_inventory;
#[path = "owner_budget/sources.rs"]
mod sources;
#[path = "owner_budget/split_layout.rs"]
mod split_layout;
#[path = "owner_budget/status_docs.rs"]
mod status_docs;
#[path = "owner_budget/virtual_geometry_debug_snapshot.rs"]
mod virtual_geometry_debug_snapshot;

fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            source.contains(anchor),
            "{label} should retain Runtime 15 performance-hotspot guard anchor `{anchor}`"
        );
    }
}

#[test]
fn runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed() {
    let sources = sources::load();
    parent_routes::assert_performance_hotspots_parent_routes(&sources);
    child_routes::assert_performance_hotspot_child_routes(&sources);
    source_inventory::assert_performance_hotpath_source_inventory(&sources);
    line_budgets::assert_performance_hotspot_guard_budgets(&sources);
    status_docs::assert_performance_hotspot_status_docs(&sources);
}

#[test]
fn runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split() {
    let sources = sources::load();
    sources::assert_sources_guard_folder_backed(&sources);
}

#[test]
fn runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_split() {
    let sources = sources::load();
    child_routes::assert_child_routes_guard_folder_backed(&sources);
}

#[test]
fn runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_split() {
    let sources = sources::load();
    line_budgets::assert_line_budgets_guard_folder_backed(&sources);
}
