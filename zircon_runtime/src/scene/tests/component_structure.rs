#[test]
fn scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scene")
        .join("components");

    for relative in ["mod.rs", "scene.rs"] {
        assert!(
            root.join(relative).exists(),
            "expected scene component module {relative} under {:?}",
            root
        );
    }

    let scene_root = root.parent().expect("scene directory exists");
    for relative in [
        "ecs/mod.rs",
        "ecs/archetype_id.rs",
        "ecs/bundle.rs",
        "ecs/component.rs",
        "ecs/component_id.rs",
        "ecs/component_registry.rs",
        "ecs/entity_location.rs",
        "ecs/entity_registry.rs",
        "ecs/internal_entity.rs",
        "ecs/resource.rs",
        "ecs/resource_id.rs",
        "ecs/resource_registry.rs",
        "ecs/schedule.rs",
        "ecs/scene_system_descriptor.rs",
        "ecs/scene_system_registry.rs",
        "ecs/storage/mod.rs",
        "ecs/storage_type.rs",
        "ecs/system_stage.rs",
    ] {
        assert!(
            scene_root.join(relative).exists(),
            "expected scene ECS module {relative} under {:?}",
            scene_root
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

#[test]
fn world_property_access_moves_into_folder_backed_subtree() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scene")
        .join("world");

    assert!(
        root.join("property_access").join("mod.rs").exists(),
        "expected world property access to move into src/scene/world/property_access/mod.rs"
    );

    for relative in [
        "property_access/path_resolution.rs",
        "property_access/entries.rs",
        "property_access/read.rs",
        "property_access/write.rs",
        "property_access/value_conversion.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected world property-access module {relative} under {:?}",
            root
        );
    }

    assert!(
        !root.join("property_access.rs").exists(),
        "flat world property_access.rs should be replaced by a folder-backed subtree"
    );
}

#[test]
fn scene_render_extract_does_not_use_snapshot_adapter_for_frame_extract() {
    let render_extract = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("render_extract")
            .join("mod.rs"),
    )
    .unwrap();

    assert!(
        !render_extract.contains("RenderFrameExtract::from_snapshot"),
        "scene render extract must populate RenderFrameExtract directly; from_snapshot is only for preview/test roundtrip adapters"
    );
}
