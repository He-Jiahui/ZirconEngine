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

#[test]
fn runtime_scene_exposes_neutral_world_inspection_surface() {
    let scene_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scene");

    assert!(
        scene_root.join("inspection").join("mod.rs").exists(),
        "runtime scene should expose neutral world inspection under src/scene/inspection"
    );
    assert!(
        !scene_root.join("editor_projection").exists(),
        "runtime scene must not keep editor_projection as a production module"
    );

    for relative in [
        "mod.rs",
        "inspection/mod.rs",
        "inspection/hierarchy.rs",
        "inspection/field.rs",
        "inspection/snapshot.rs",
    ] {
        let source = std::fs::read_to_string(scene_root.join(relative)).unwrap();
        assert!(
            !source.contains("SceneEditor") && !source.contains("editor_projection"),
            "runtime scene inspection public surface must stay neutral in {relative}"
        );
    }
}

#[test]
fn scene_project_serialization_sources_do_not_store_editor_authoring_state() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    for relative in [
        "src/scene/world/world.rs",
        "src/scene/world/project_io.rs",
        "src/scene/dynamic_scene/document.rs",
        "src/scene/dynamic_scene/entity.rs",
        "src/scene/dynamic_scene/scene.rs",
        "src/scene/dynamic_scene/value.rs",
        "src/asset/assets/scene.rs",
    ] {
        let path = manifest_root.join(relative);
        let source = std::fs::read_to_string(&path).unwrap();

        for forbidden in [
            "selected",
            "selection",
            "selected_entity",
            "selected_node",
            "set_selected",
            "SceneViewportSettings",
            "SceneViewportTool",
            "TransformSpace",
            "GridMode",
            "ViewOrientation",
            "RenderOverlayExtract",
            "SelectionHighlightExtract",
            "SelectionAnchorExtract",
            "GridOverlayExtract",
            "HandleOverlayExtract",
            "SceneGizmoOverlayExtract",
            "SceneGizmoKind",
            "scene_gizmos",
            "selection_anchors",
            "active_camera_override",
            "camera_override",
            "preview_lighting",
            "preview_skybox",
            "display_mode",
        ] {
            assert!(
                !source.contains(forbidden),
                "runtime scene project serialization source {relative} must not store editor authoring state token {forbidden}"
            );
        }
    }
}

#[test]
fn scene_ecs_does_not_reintroduce_late_update_stage_or_compatibility_path() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    for relative in ["src/scene/ecs", "src/scene/module", "src/scene/world"] {
        assert_no_legacy_late_update_name(&manifest_root.join(relative));
    }
}

fn assert_no_legacy_late_update_name(root: &std::path::Path) {
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            assert_no_legacy_late_update_name(&path);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let source = std::fs::read_to_string(&path).unwrap();
        assert!(
            !source.contains("LateUpdate"),
            "scene ECS scheduling must not reintroduce LateUpdate aliases, shims, compatibility stages, or re-export bridges in {:?}",
            path
        );
    }
}
