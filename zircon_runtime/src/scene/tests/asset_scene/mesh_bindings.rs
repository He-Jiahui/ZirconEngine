use super::*;

#[test]
fn scene_assets_instantiate_world_with_asset_bound_meshes() {
    let root = unique_temp_project_root("scene_asset");
    let project = create_test_project(&root);
    let world = World::load_scene_from_uri(
        &project,
        &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
    )
    .unwrap();

    let mesh_node = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Mesh))
        .unwrap();
    let mesh = mesh_node.mesh.as_ref().unwrap();
    assert_eq!(
        mesh.model,
        project_model_handle(&project, "res://models/triangle.obj")
    );
    assert_eq!(
        mesh.mesh,
        Some(project_mesh_handle(&project, "res://meshes/triangle.zmesh"))
    );
    assert_eq!(
        mesh.material,
        project_material_handle(&project, "res://materials/grid.zmaterial")
    );
    assert!(mesh.primitives.is_empty());

    let saved = world.to_scene_asset(&project).unwrap();
    assert_scene_asset_excludes_authoring_tokens("scene asset JSON", &saved);
    let saved_mesh = saved
        .entities
        .iter()
        .find_map(|entity| entity.mesh.as_ref())
        .unwrap();
    assert_eq!(saved_mesh.model.to_string(), "res://models/triangle.obj");
    assert_eq!(
        saved_mesh.mesh.as_ref().map(ToString::to_string),
        Some("res://meshes/triangle.zmesh".to_string())
    );
    assert_eq!(
        saved_mesh.material.to_string(),
        "res://materials/grid.zmaterial"
    );
    assert!(saved_mesh.primitives.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_extract_keeps_asset_bound_meshes_without_editor_selection_overlay() {
    let root = unique_temp_project_root("scene_gizmo");
    let project = create_test_project(&root);
    let world = World::load_scene_from_uri(
        &project,
        &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
    )
    .unwrap();
    let mesh_node = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Mesh))
        .unwrap()
        .id;

    let extract = world.to_render_extract();
    let mesh = extract
        .scene
        .meshes
        .iter()
        .find(|mesh| mesh.node_id == mesh_node)
        .unwrap();
    assert_eq!(
        mesh.model,
        project_model_handle(&project, "res://models/triangle.obj")
    );
    assert_eq!(
        mesh.mesh,
        Some(project_mesh_handle(&project, "res://meshes/triangle.zmesh"))
    );
    assert_eq!(
        mesh.material,
        project_material_handle(&project, "res://materials/grid.zmaterial")
    );
    assert!(extract.overlays.selection.is_empty());
    assert!(extract
        .scene
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == mesh_node));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scene_assets_roundtrip_primitive_mesh_material_bindings() {
    let root = unique_temp_project_root("scene_primitive_bindings");
    let project = create_test_project(&root);
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 77,
            name: "PrimitiveBindings".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 0x0000_0001,
            mobility: SceneMobilityAsset::Dynamic,
            camera: None,
            mesh: Some(SceneMeshInstanceAsset {
                model: asset_reference("res://models/triangle.obj"),
                mesh: None,
                material: asset_reference("res://materials/grid.zmaterial"),
                render_queue: 2_450,
                material_queue: -12,
                order_in_layer: 12,
                depth_bias: -0.5,
                morph_weights: vec![0.25, 1.0],
                primitives: vec![SceneMeshPrimitiveBindingAsset {
                    mesh: asset_reference("res://meshes/triangle.zmesh"),
                    material: asset_reference("res://materials/grid.zmaterial"),
                }],
                lods: vec![SceneMeshLodLevelAsset {
                    min_distance: 18.0,
                    model: asset_reference("res://models/triangle.obj"),
                    mesh: Some(asset_reference("res://meshes/triangle.zmesh")),
                    material: asset_reference("res://materials/grid.zmaterial"),
                    primitives: Vec::new(),
                }],
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
        }],
    };

    let world = World::from_scene_asset(&project, &scene).unwrap();
    let mesh_node = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Mesh))
        .unwrap();
    let mesh = mesh_node.mesh.as_ref().unwrap();
    assert_eq!(mesh.render_queue, 2_450);
    assert_eq!(mesh.material_queue, -12);
    assert_eq!(mesh.order_in_layer, 12);
    assert_eq!(mesh.depth_bias, -0.5);
    assert_eq!(mesh.morph_weights, vec![0.25, 1.0]);
    assert_eq!(mesh.primitives.len(), 1);
    assert_eq!(
        mesh.primitives[0].mesh,
        project_mesh_handle(&project, "res://meshes/triangle.zmesh")
    );
    assert_eq!(
        mesh.primitives[0].material,
        project_material_handle(&project, "res://materials/grid.zmaterial")
    );
    assert_eq!(mesh.lods.len(), 1);
    assert_eq!(mesh.lods[0].min_distance, 18.0);
    assert_eq!(
        mesh.lods[0].model,
        project_model_handle(&project, "res://models/triangle.obj")
    );
    assert_eq!(
        mesh.lods[0].mesh,
        Some(project_mesh_handle(&project, "res://meshes/triangle.zmesh"))
    );
    assert_eq!(
        mesh.lods[0].material,
        project_material_handle(&project, "res://materials/grid.zmaterial")
    );

    let extract = world.to_render_extract();
    let render_mesh = extract
        .scene
        .meshes
        .iter()
        .find(|mesh| mesh.node_id == mesh_node.id)
        .unwrap();
    assert_eq!(render_mesh.mesh, Some(mesh.primitives[0].mesh));
    assert_eq!(render_mesh.material, mesh.primitives[0].material);
    assert_eq!(render_mesh.morph_weights, vec![0.25, 1.0]);

    let saved = world.to_scene_asset(&project).unwrap();
    let saved_mesh = saved.entities[0].mesh.as_ref().unwrap();
    assert_eq!(saved_mesh.render_queue, 2_450);
    assert_eq!(saved_mesh.material_queue, -12);
    assert_eq!(saved_mesh.order_in_layer, 12);
    assert_eq!(saved_mesh.depth_bias, -0.5);
    assert_eq!(saved_mesh.morph_weights, vec![0.25, 1.0]);
    let saved_binding = &saved_mesh.primitives[0];
    assert_eq!(
        saved_binding.mesh.to_string(),
        "res://meshes/triangle.zmesh"
    );
    assert_eq!(
        saved_binding.material.to_string(),
        "res://materials/grid.zmaterial"
    );
    let saved_lod = &saved_mesh.lods[0];
    assert_eq!(saved_lod.min_distance, 18.0);
    assert_eq!(saved_lod.model.to_string(), "res://models/triangle.obj");
    assert_eq!(
        saved_lod.mesh.as_ref().map(ToString::to_string),
        Some("res://meshes/triangle.zmesh".to_string())
    );
    assert_eq!(
        saved_lod.material.to_string(),
        "res://materials/grid.zmaterial"
    );

    let _ = fs::remove_dir_all(root);
}
