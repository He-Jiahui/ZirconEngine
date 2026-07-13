use super::*;

#[test]
fn artifact_store_roundtrips_scene_assets_with_mesh_references() {
    let root = unique_temp_project_root("artifact_store_scene_mesh_references");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 338_863_232_448_440,
            name: "fence-gate".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 1,
            mobility: SceneMobilityAsset::Static,
            camera: None,
            mesh: Some(crate::asset::SceneMeshInstanceAsset {
                model: asset_reference("res://models/kenney_graveyard/fence-gate.glb#Mesh0"),
                mesh: None,
                material: asset_reference("res://models/kenney_graveyard/fence-gate.glb#Material0"),
                render_queue: 0,
                material_queue: 0,
                order_in_layer: 0,
                depth_bias: 0.0,
                morph_weights: Vec::new(),
                primitives: vec![crate::asset::SceneMeshPrimitiveBindingAsset {
                    mesh: asset_reference(
                        "res://models/kenney_graveyard/fence-gate.glb#Mesh0/Primitive0",
                    ),
                    material: asset_reference(
                        "res://models/kenney_graveyard/fence-gate.glb#Material0",
                    ),
                }],
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
        }],
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Scene,
        AssetUri::parse("res://models/kenney_graveyard/fence-gate.glb#Scene0").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Scene(scene.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Scene(scene));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_scene_assets_with_camera_targets() {
    let root = unique_temp_project_root("artifact_store_scene_camera_targets");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 1,
            name: "RenderTargetCamera".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 1,
            mobility: SceneMobilityAsset::Dynamic,
            camera: Some(SceneCameraAsset {
                target: SceneCameraTargetAsset::Texture {
                    texture: asset_reference("res://textures/reflection_target.texture.toml"),
                },
                ..SceneCameraAsset::default()
            }),
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
        }],
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Scene,
        AssetUri::parse("res://scenes/render_target.scene.toml").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Scene(scene.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Scene(scene));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_scene_assets_with_physics_components() {
    let root = unique_temp_project_root("artifact_store_scene_physics_components");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 2,
            name: "PhysicsDoor".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 1,
            mobility: SceneMobilityAsset::Dynamic,
            camera: None,
            mesh: None,
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            post_process_volume: None,
            rigid_body: Some(SceneRigidBodyAsset {
                body_type: SceneRigidBodyTypeAsset::Dynamic,
                mass: 4.5,
                mass_properties: PhysicsMassProperties::AutoFromShape { density: 2.25 },
                linear_velocity: [0.5, 0.0, 0.0],
                angular_velocity: [0.0, 0.25, 0.0],
                linear_damping: 0.1,
                angular_damping: 0.2,
                gravity_scale: 1.0,
                ccd_mode: Default::default(),
                sleep_policy: Default::default(),
                lock_translation: [false, false, false],
                lock_rotation: [false, true, false],
            }),
            collider: Some(SceneColliderAsset {
                shape: SceneColliderShapeAsset::Capsule {
                    radius: 0.35,
                    half_height: 1.1,
                },
                sensor: false,
                layer: 2,
                collision_group: 3,
                collision_mask: 0b101,
                material: Some(asset_reference(
                    "res://physics/materials/hinge.physics_material.toml",
                )),
                material_override: Some(PhysicsMaterialMetadata {
                    static_friction: 0.8,
                    dynamic_friction: 0.6,
                    restitution: 0.1,
                    ..PhysicsMaterialMetadata::default()
                }),
                local_transform: TransformAsset::default(),
            }),
            joint: Some(SceneJointAsset {
                joint_type: SceneJointKindAsset::Generic6Dof,
                connected_entity: Some(1),
                anchor: [0.0, 1.0, 0.0],
                axis: [0.0, 1.0, 0.0],
                limits: Some([-0.5, 0.5]),
                collide_connected: true,
                constraint: PhysicsJointConstraintMetadata {
                    linear_limits: [Some([-0.2, 0.2]), None, Some([0.0, 1.0])],
                    angular_limits: [None, Some([-0.25, 0.25]), None],
                    break_force: Some(120.0),
                    ..PhysicsJointConstraintMetadata::default()
                },
                skeleton_binding: None,
            }),
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
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Scene,
        AssetUri::parse("res://scenes/physics.scene.toml").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Scene(scene.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Scene(scene));

    let _ = fs::remove_dir_all(root);
}
