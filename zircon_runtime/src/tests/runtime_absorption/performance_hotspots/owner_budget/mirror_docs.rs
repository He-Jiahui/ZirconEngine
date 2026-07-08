#[path = "mirror_docs/audit_wiring.rs"]
mod audit_wiring;
#[path = "mirror_docs/doc_mirrors.rs"]
mod doc_mirrors;
#[path = "mirror_docs/performance_guard.rs"]
mod performance_guard;
#[path = "mirror_docs/source_inventory.rs"]
mod source_inventory;
#[path = "mirror_docs/sources.rs"]
mod sources;
#[path = "mirror_docs/split_layout.rs"]
mod split_layout;

#[test]
fn runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts() {
    let sources = sources::load();
    performance_guard::assert_performance_guard_anchors(&sources);
    source_inventory::assert_source_inventory_anchors(&sources);
    audit_wiring::assert_audit_wiring_anchors(&sources);
    doc_mirrors::assert_runtime_07_mirror_docs(&sources);
    split_layout::assert_mirror_docs_split_layout(&sources);
}

#[test]
fn runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_split() {
    let sources = sources::load();
    split_layout::assert_mirror_docs_split_layout(&sources);
}
