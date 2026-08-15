#[test]
fn scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scene")
        .join("components");

    for relative in ["mod.rs", "scene/mod.rs"] {
        assert!(
            root.join(relative).exists(),
            "expected scene component module {relative} under {:?}",
            root
        );
    }

    let scene_root = root.parent().expect("scene directory exists");
    for relative in [
        "ecs/mod.rs",
        "ecs/archetype/mod.rs",
        "ecs/archetype/id.rs",
        "ecs/archetype/index.rs",
        "ecs/archetype/record.rs",
        "ecs/archetype/signature.rs",
        "ecs/archetype/table/mod.rs",
        "ecs/archetype/table/column.rs",
        "ecs/archetype/table/error.rs",
        "ecs/archetype/table/preflighted_row.rs",
        "ecs/archetype/table/table.rs",
        "ecs/archetype/table/taken_row.rs",
        "ecs/bundle.rs",
        "ecs/commands/mod.rs",
        "ecs/commands/command.rs",
        "ecs/commands/command_queue.rs",
        "ecs/commands/commands/mod.rs",
        "ecs/commands/commands/entity_commands.rs",
        "ecs/commands/commands/facade.rs",
        "ecs/commands/commands/param.rs",
        "ecs/change_detection/mod.rs",
        "ecs/change_detection/change_tick.rs",
        "ecs/change_detection/change_tick_window.rs",
        "ecs/change_detection/component_ticks.rs",
        "ecs/change_detection/stats.rs",
        "ecs/change_detection/wrappers.rs",
        "ecs/component/mod.rs",
        "ecs/component/id.rs",
        "ecs/component/marker.rs",
        "ecs/component/registry.rs",
        "ecs/entity/mod.rs",
        "ecs/entity/despawned.rs",
        "ecs/entity/error.rs",
        "ecs/entity/internal.rs",
        "ecs/entity/location.rs",
        "ecs/entity/registry.rs",
        "ecs/entity/slot.rs",
        "ecs/entity/stable_location.rs",
        "ecs/resource/mod.rs",
        "ecs/resource/id.rs",
        "ecs/resource/marker.rs",
        "ecs/resource/registry.rs",
        "ecs/observer/mod.rs",
        "ecs/observer/callbacks.rs",
        "ecs/observer/callback_registry.rs",
        "ecs/observer/entry.rs",
        "ecs/observer/id.rs",
        "ecs/observer/store.rs",
        "ecs/schedule.rs",
        "ecs/scene_system_descriptor.rs",
        "ecs/scene_system_registry.rs",
        "ecs/storage/mod.rs",
        "ecs/storage/component_storage/mod.rs",
        "ecs/storage/component_storage/component_results.rs",
        "ecs/storage/component_storage/entry.rs",
        "ecs/storage/component_storage/location.rs",
        "ecs/storage/component_storage/sparse.rs",
        "ecs/storage/component_storage/store.rs",
        "ecs/storage_type.rs",
    ] {
        assert!(
            scene_root.join(relative).exists(),
            "expected scene ECS module {relative} under {:?}",
            scene_root
        );
    }

    let framework_scene_root = scene_root
        .parent()
        .expect("src directory exists")
        .join("core")
        .join("framework")
        .join("scene");
    assert!(
        framework_scene_root.join("system_stage.rs").exists(),
        "SystemStage should stay owned by core/framework/scene after the hard cutover"
    );
    assert!(
        !scene_root.join("ecs/system_stage.rs").exists(),
        "retired scene/ecs/system_stage.rs must not return as an alias or compatibility owner"
    );
    let ecs_root_source = std::fs::read_to_string(scene_root.join("ecs/mod.rs")).unwrap();
    assert!(
        ecs_root_source.contains("pub use crate::core::framework::scene::SystemStage;"),
        "scene/ecs should route SystemStage from its core/framework owner"
    );

    for relative in ["render_extract.rs", "viewport.rs", "gizmo.rs"] {
        assert!(
            !root.join(relative).exists(),
            "editor-owned scene authoring module {relative} should not live under {:?}",
            root
        );
    }
}

#[test]
fn scene_component_owner_tree_stays_domain_split_without_active_alias() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scene")
        .join("components");
    let route = std::fs::read_to_string(root.join("scene/mod.rs")).unwrap();
    assert!(
        !root.join("scene.rs").exists(),
        "the retired scene.rs forwarding shell must not return"
    );
    for declaration_keyword in ["struct", "enum", "type", "fn", "impl"] {
        assert!(
            !contains_rust_identifier(&route, declaration_keyword),
            "scene/mod.rs must stay a declaration-free structural route; found {declaration_keyword}"
        );
    }

    let owner_declarations: &[(&str, &[&str])] = &[
        (
            "activation.rs",
            &[
                "pub struct ActiveSelf",
                "pub struct ActiveInHierarchy",
                "pub struct RenderLayerMask",
                "pub const fn default_render_layer_mask",
            ],
        ),
        (
            "animation.rs",
            &[
                "pub struct AnimationSkeletonComponent",
                "pub struct AnimationPlayerComponent",
                "pub struct AnimationSequencePlayerComponent",
                "pub struct AnimationGraphPlayerComponent",
                "pub struct AnimationStateMachinePlayerComponent",
            ],
        ),
        ("camera.rs", &["pub struct CameraComponent"]),
        ("hierarchy.rs", &["pub struct Hierarchy"]),
        ("identity.rs", &["pub enum NodeKind", "pub struct Name"]),
        (
            "mesh_renderer.rs",
            &[
                "pub struct MeshRendererPrimitiveBinding",
                "pub struct MeshRendererLodLevel",
                "pub struct MeshRenderer",
            ],
        ),
        (
            "node.rs",
            &["pub struct SceneNode", "pub struct NodeRecord"],
        ),
        (
            "physics.rs",
            &[
                "pub enum RigidBodyType",
                "pub struct RigidBodyComponent",
                "pub enum ColliderShape",
                "pub struct ColliderComponent",
                "pub enum JointKind",
                "pub struct JointComponent",
            ],
        ),
        (
            "transform.rs",
            &[
                "pub struct LocalTransform",
                "pub struct WorldMatrix",
                "pub struct WorldTransform",
            ],
        ),
    ];
    for &(owner, declarations) in owner_declarations {
        let owner_path = root.join("scene").join(owner);
        let source = std::fs::read_to_string(&owner_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", owner_path.display()));
        for declaration in declarations {
            assert!(
                source.contains(declaration),
                "scene component domain owner {owner} must retain {declaration}"
            );
        }
    }

    assert_no_rust_identifier_in_tree(&root, "Active");
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
    // Keep the retired editor-projection module name only as a resurrection guard.
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

fn assert_no_rust_identifier_in_tree(root: &std::path::Path, identifier: &str) {
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            assert_no_rust_identifier_in_tree(&path, identifier);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let source = std::fs::read_to_string(&path).unwrap();
        assert!(
            !contains_rust_identifier(&source, identifier),
            "retired Rust identifier {identifier} must not return in {:?}",
            path
        );
    }
}

fn contains_rust_identifier(source: &str, identifier: &str) -> bool {
    source
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token == identifier)
}
