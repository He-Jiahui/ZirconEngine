#[test]
fn runtime_navigation_boundary_file_set_requires_doc_update() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/navigation");
    super::navigation_inventory::assert_navigation_inventory(&root);
    let registration = include_str!("../../../../navigation/operation/registration.rs");
    assert!(registration.contains("#[path = \"history_retention.rs\"]"));
    assert!(registration.contains("pub(super) mod history_retention;"));
    let handler = include_str!("../../../../navigation/operation/handler.rs");
    assert!(handler.contains("#[cfg(test)]"));
    assert!(handler.contains("#[path = \"tests/handler.rs\"]"));
    assert!(handler.contains("super::registration::history_retention::preflight_snapshots("));
    assert!(handler.contains("super::registration::history_retention::certify_result("));
    for current_doc in [
        include_str!("../../../../../../docs/crates/zircon_runtime/navigation/runtime.md"),
        include_str!("../../../../../../docs/architecture/runtime-root-surface-m1.md"),
    ] {
        for owner in [
            "operation/history_retention.rs",
            "operation/tests/handler.rs",
            "pre-publication history admission",
        ] {
            assert!(
                current_doc.contains(owner),
                "current navigation responsibility: {owner}"
            );
        }
    }
    let module_source = include_str!("../../../../navigation/module.rs");
    assert!(
        module_source
            .contains("Built-in baked navmesh pathfinding and lightweight agent avoidance"),
        "navigation module descriptor should keep fallback runtime scope explicit"
    );
    let boundary_doc =
        include_str!("../../../../../../docs/crates/zircon_runtime/navigation/runtime.md");
    for required_anchor in [
        "Runtime 14 Boundary Judgment",
        "built-in fallback implementation",
        "folder-backed runtime owner split",
        "runtime/avoidance.rs",
        "runtime/baked_mesh.rs",
        "runtime/world_scan.rs",
        "operation/handler.rs",
        "operation/registration.rs",
        "zircon_plugins/navigation",
        "runtime_navigation_boundary_file_set_requires_doc_update",
    ] {
        assert!(boundary_doc.contains(required_anchor), "{required_anchor}");
    }
}
