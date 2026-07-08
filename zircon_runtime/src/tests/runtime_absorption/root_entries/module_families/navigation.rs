#[test]
fn runtime_navigation_boundary_file_set_requires_doc_update() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let navigation_dir = manifest_dir.join("src").join("navigation");
    let actual_entries = std::fs::read_dir(&navigation_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", navigation_dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("failed to read navigation entry: {error}"))
                .file_name()
                .into_string()
                .unwrap_or_else(|name| panic!("non-utf8 navigation entry name: {name:?}"))
        })
        .collect::<std::collections::BTreeSet<_>>();
    let expected_entries = ["mod.rs", "module.rs", "runtime", "runtime.rs"]
        .into_iter()
        .map(String::from)
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        actual_entries, expected_entries,
        "runtime navigation fallback changed root file shape; update docs/zircon_runtime/navigation/runtime.md and Runtime 14 before adding behavior files"
    );

    let runtime_dir = navigation_dir.join("runtime");
    let actual_runtime_entries = std::fs::read_dir(&runtime_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", runtime_dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("failed to read navigation runtime entry: {error}"))
                .file_name()
                .into_string()
                .unwrap_or_else(|name| panic!("non-utf8 navigation runtime entry name: {name:?}"))
        })
        .collect::<std::collections::BTreeSet<_>>();
    let expected_runtime_entries = [
        "avoidance.rs",
        "baked_mesh.rs",
        "math.rs",
        "state.rs",
        "tests.rs",
        "world_scan.rs",
    ]
    .into_iter()
    .map(String::from)
    .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        actual_runtime_entries, expected_runtime_entries,
        "runtime navigation fallback changed owner modules; update docs/zircon_runtime/navigation/runtime.md and Runtime 14 before adding behavior files"
    );

    let module_source = include_str!("../../../../navigation/module.rs");
    assert!(
        module_source
            .contains("Built-in baked navmesh pathfinding and lightweight agent avoidance"),
        "navigation module descriptor should keep fallback runtime scope explicit"
    );

    let boundary_doc = include_str!("../../../../../../docs/zircon_runtime/navigation/runtime.md");
    for required_anchor in [
        "Runtime 14 Boundary Judgment",
        "built-in fallback implementation",
        "folder-backed runtime owner split",
        "runtime/avoidance.rs",
        "runtime/baked_mesh.rs",
        "runtime/world_scan.rs",
        "zircon_plugins/navigation",
        "runtime_navigation_boundary_file_set_requires_doc_update",
    ] {
        assert!(
            boundary_doc.contains(required_anchor),
            "navigation boundary doc should record `{required_anchor}`"
        );
    }
}
