#[test]
fn scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("components");

    for relative in ["mod.rs", "schedule.rs", "scene.rs"] {
        assert!(
            root.join(relative).exists(),
            "expected scene component module {relative} under {:?}",
            root
        );
    }

    for relative in ["render_extract.rs", "viewport.rs", "gizmo.rs"] {
        assert!(
            !root.join(relative).exists(),
            "editor-owned scene authoring module {relative} should not live under {:?}",
            root
        );
    }
}
