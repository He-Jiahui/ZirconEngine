use super::*;

#[test]
fn scene_asset_save_reopens_with_exact_transform_and_asset_references() {
    let root = unique_temp_project_root("scene_asset_save_reopen");
    let project = create_test_project(&root);
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let mut world = World::load_scene_from_uri(&project, &scene_uri).unwrap();
    let expected_transform = Transform::from_translation(Vec3::new(3.5, 0.0, -1.25));

    assert!(world.update_transform(2, expected_transform).unwrap());
    let expected_scene = world.to_scene_asset(&project).unwrap();

    world.save_scene_to_project(&project, &scene_uri).unwrap();
    let scene_path = project
        .existing_or_primary_project_source_path_for_uri(&scene_uri)
        .unwrap();
    let first_save = fs::read_to_string(&scene_path).unwrap();
    world.save_scene_to_project(&project, &scene_uri).unwrap();
    assert_eq!(fs::read_to_string(&scene_path).unwrap(), first_save);

    drop(project);

    let mut reopened_project = crate::asset::project::ProjectManager::open(&root).unwrap();
    reopened_project
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    reopened_project.scan_and_import().unwrap();
    let reopened_world = World::load_scene_from_uri(&reopened_project, &scene_uri).unwrap();

    assert_eq!(reopened_world.to_scene_asset(&reopened_project).unwrap(), expected_scene);
    assert_eq!(
        reopened_world.find_node(2).unwrap().transform,
        expected_transform
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn unresolved_scene_reference_returns_typed_dangling_error() {
    let root = unique_temp_project_root("scene_dangling_reference");
    let project = create_test_project(&root);
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let ImportedAsset::Scene(mut scene) = project.load_artifact(&scene_uri).unwrap() else {
        panic!("fixture should import a scene");
    };
    let missing_uuid = AssetUuid::new();
    let missing_uri = AssetUri::parse("res://models/missing.obj").unwrap();
    scene
        .entities
        .iter_mut()
        .find_map(|entity| entity.mesh.as_mut())
        .expect("fixture scene mesh")
        .model = AssetReference::new(missing_uuid, missing_uri.clone());

    let error = World::from_scene_asset(&project, &scene).unwrap_err();

    assert!(matches!(
        error,
        SceneProjectError::DanglingAssetReference { uuid, locator }
            if uuid == missing_uuid && locator == missing_uri
    ));
    let _ = fs::remove_dir_all(root);
}

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
