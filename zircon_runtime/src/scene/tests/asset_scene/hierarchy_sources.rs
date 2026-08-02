use super::*;

#[test]
fn scene_assets_keep_script_only_entities_as_empty_nodes() {
    let root = unique_temp_project_root("scene_script_bindings");
    let project = create_test_project(&root);
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 90,
            name: "Player".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 0x0000_0001,
            mobility: SceneMobilityAsset::Dynamic,
            camera: None,
            mesh: None,
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            post_process_volume: None,
            rigid_body: None,
            collider: None,
            joint: None,
            animation_skeleton: None,
            animation_player: None,
            animation_sequence_player: None,
            animation_graph_player: None,
            animation_state_machine_player: None,
            terrain: None,
            tilemap: None,
            prefab_instance: None,
            script_bindings: vec![SceneScriptBindingAsset {
                package: "vampire_game".to_string(),
                module: "player".to_string(),
                enabled: true,
                update: true,
                fixed_update: false,
                properties: std::collections::BTreeMap::new(),
            }],
        }],
    };

    let world = World::from_scene_asset(&project, &scene).unwrap();
    let player = world
        .nodes()
        .iter()
        .find(|node| node.name == "Player")
        .unwrap();

    assert!(matches!(player.kind, NodeKind::Empty));
    assert!(world.dynamic_component(90, "script.bindings").is_some());
    let extract = world.to_render_extract();
    assert!(extract.scene.meshes.is_empty());
    assert!(extract.scene.directional_lights.is_empty());
    assert_eq!(world.to_scene_asset(&project).unwrap(), scene);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scene_asset_load_uses_asset_preserving_normalizer_source_guard() {
    let source = project_io_source();
    let from_scene_asset =
        project_io_section(source, "pub fn from_scene_asset", "pub fn to_scene_asset");
    assert!(from_scene_asset.contains("world.normalize_scene_asset_after_load();"));
    assert!(!from_scene_asset.contains("world.normalize_after_load();"));

    let scene_asset_normalizer = project_io_section(
        source,
        "fn normalize_scene_asset_after_load",
        "fn normalize_after_load",
    );
    assert!(scene_asset_normalizer.contains("self.normalize_loaded_state(false);"));

    let project_normalizer = project_io_section(
        source,
        "fn normalize_after_load",
        "fn normalize_loaded_state",
    );
    assert!(project_normalizer.contains("self.normalize_loaded_state(true);"));

    let normalize_loaded_state = project_io_section(
        source,
        "fn normalize_loaded_state",
        "self.flush_scene_systems_now();",
    );
    assert!(normalize_loaded_state.contains("if ensure_default_nodes && self.cameras.is_empty()"));
    assert!(normalize_loaded_state.contains("self.spawn_node(NodeKind::Camera);"));
    assert!(
        normalize_loaded_state
            .contains("if ensure_default_nodes && self.directional_lights.is_empty()")
    );
    assert!(normalize_loaded_state.contains("self.spawn_node(NodeKind::DirectionalLight);"));
    assert!(!normalize_loaded_state.contains("if self.cameras.is_empty()"));
    assert!(!normalize_loaded_state.contains("if self.directional_lights.is_empty()"));
}

#[test]
fn project_persistence_borrows_world_and_normalizes_without_entity_snapshot() {
    let source = project_io_source();
    let save = project_io_section(
        source,
        "pub fn save_project_to_path",
        "pub fn load_project_from_path",
    );
    let normalize = project_io_section(
        source,
        "fn normalize_loaded_state",
        "self.flush_scene_systems_now();",
    );

    assert!(
        source.contains("struct ProjectDocumentRef<'world>")
            && source.contains("world: &'world World,")
            && save.contains("let document = ProjectDocumentRef {")
            && save.contains("world: self,")
            && !save.contains("world: self.clone(),"),
        "project save must serialize a borrowed World instead of cloning the entire scene"
    );
    assert!(
        normalize.contains("for entity_index in 0..self.entities.len()")
            && normalize.contains("let entity = self.entities[entity_index];")
            && !normalize.contains("self.entities.iter().copied().collect::<Vec<_>>()"),
        "load normalization must index the stable entity list without allocating an id snapshot"
    );
}

#[test]
fn builtin_reference_locators_are_parsed_once() {
    let source = include_str!("../../world/project_io/references.rs");
    let lookup = source
        .split("fn builtin_locator_for_id(")
        .nth(1)
        .expect("read builtin locator lookup");

    assert!(
        source.contains("static BUILTIN_LOCATORS: OnceLock<")
            && source.contains("BUILTIN_LOCATORS.get_or_init(")
            && lookup.contains("for (candidate_id, locator) in builtin_locators()")
            && !lookup.contains("ResourceLocator::parse(locator_text)"),
        "builtin resource locators and ids must be parsed once instead of once per saved handle"
    );
}

#[test]
fn scene_assets_keep_transform_only_hierarchy_nodes() {
    let root = unique_temp_project_root("scene_empty_hierarchy");
    let project = create_test_project(&root);
    let scene = SceneAsset {
        entities: vec![
            SceneEntityAsset {
                entity: 10,
                name: "ActorRoot".to_string(),
                parent: None,
                transform: TransformAsset::default(),
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: None,
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
                script_bindings: Vec::new(),
            },
            SceneEntityAsset {
                entity: 11,
                name: "ActorMesh".to_string(),
                parent: Some(10),
                transform: TransformAsset::default(),
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: asset_reference("res://models/triangle.obj"),
                    mesh: Some(asset_reference("res://meshes/triangle.zmesh")),
                    material: asset_reference("res://materials/grid.zmaterial"),
                    render_queue: 0,
                    material_queue: 0,
                    order_in_layer: 0,
                    depth_bias: 0.0,
                    morph_weights: Vec::new(),
                    primitives: Vec::new(),
                    lods: Vec::new(),
                }),
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
                script_bindings: Vec::new(),
            },
        ],
    };

    let world = World::from_scene_asset(&project, &scene).unwrap();
    let root_node = world.find_node(10).expect("transform-only root node");
    assert!(matches!(root_node.kind, NodeKind::Empty));
    assert_eq!(world.parent_of(11), Some(10));
    let saved = world.to_scene_asset(&project).unwrap();
    assert!(
        saved
            .entities
            .iter()
            .any(|entity| entity.entity == 10 && entity.mesh.is_none())
    );
    assert!(
        saved
            .entities
            .iter()
            .any(|entity| entity.entity == 11 && entity.parent == Some(10))
    );

    let _ = fs::remove_dir_all(root);
}
