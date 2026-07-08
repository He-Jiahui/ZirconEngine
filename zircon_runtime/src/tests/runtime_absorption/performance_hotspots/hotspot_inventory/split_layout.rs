#[path = "split_layout/route.rs"]
mod route;
#[path = "split_layout/source_inventory.rs"]
mod source_inventory;
#[path = "split_layout/sources.rs"]
mod sources;
#[path = "split_layout/status_docs.rs"]
mod status_docs;

#[test]
fn runtime_15_runtime_07_hotspot_inventory_guard_child_owner_split() {
    let sources = sources::load();
    route::assert_hotspot_inventory_split_route(&sources);
    source_inventory::assert_hotspot_inventory_source_inventory(&sources);
    status_docs::assert_hotspot_inventory_status_docs(&sources);
}
