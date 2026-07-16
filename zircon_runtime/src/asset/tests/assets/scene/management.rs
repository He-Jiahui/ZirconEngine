use crate::asset::{
    AssetReference, AssetUri, AssetUuid, PrefabInstanceAsset, SceneAnimationGraphPlayerAsset,
    SceneAnimationPlayerAsset, SceneAsset, SceneAssetManagementRecord,
    SceneAssetManagementRecordSet, SceneAssetOverview, SceneCameraAsset, SceneCameraTargetAsset,
    SceneColliderAsset, SceneColliderShapeAsset, SceneDirectionalLightAsset, SceneEntityAsset,
    SceneEntityManagementRecordSet, SceneJointAsset, SceneJointKindAsset, SceneMeshInstanceAsset,
    SceneMobilityAsset, ScenePointLightAsset, SceneRigidBodyAsset, SceneRigidBodyTypeAsset,
    SceneTerrainAsset, SceneTileMapAsset, TransformAsset,
};
use crate::core::resource::ResourceId;

#[test]
fn scene_asset_overview_reports_entity_component_and_reference_counts() {
    let camera_target = asset_ref(
        "camera-target-overview",
        "res://textures/camera-target-overview.png",
    );
    let model = asset_ref("hero-model-overview", "res://models/hero-overview.gltf");
    let mesh = asset_ref(
        "hero-mesh-overview",
        "res://models/hero-overview.gltf#Mesh0/Primitive0",
    );
    let material = asset_ref(
        "hero-material-overview",
        "res://materials/hero-overview.zmaterial",
    );
    let physics_material = asset_ref(
        "hero-physics-material-overview",
        "res://physics/hero-overview.physics_material.toml",
    );
    let animation_graph = asset_ref(
        "hero-graph-overview",
        "res://animation/hero-overview.graph.zranim",
    );
    let terrain = asset_ref(
        "hero-terrain-overview",
        "res://terrain/hero-overview.zterrain",
    );
    let tilemap = asset_ref(
        "hero-tilemap-overview",
        "res://tilemaps/hero-overview.ztilemap",
    );
    let prefab = asset_ref(
        "hero-prefab-overview",
        "res://prefabs/hero-overview.zprefab",
    );

    let mut camera_entity = empty_scene_entity(10, "Camera");
    camera_entity.camera = Some(SceneCameraAsset {
        target: SceneCameraTargetAsset::Texture {
            texture: camera_target.clone(),
        },
        post_process_settings: None,
        ..SceneCameraAsset::default()
    });

    let mut hero_entity = empty_scene_entity(11, "Hero");
    hero_entity.parent = Some(10);
    hero_entity.active = false;
    hero_entity.render_layer_mask = 0x0000_0010;
    hero_entity.mobility = SceneMobilityAsset::Static;
    hero_entity.mesh = Some(SceneMeshInstanceAsset {
        model: model.clone(),
        mesh: Some(mesh.clone()),
        material: material.clone(),
        render_queue: 0,
        material_queue: 0,
        order_in_layer: 0,
        depth_bias: 0.0,
        morph_weights: vec![0.25, 0.75],
        primitives: Vec::new(),
        lods: Vec::new(),
    });
    hero_entity.point_light = Some(ScenePointLightAsset {
        color: [1.0, 0.8, 0.6],
        intensity: 4.0,
        range: 12.0,
        volumetric: false,
    });
    hero_entity.rigid_body = Some(SceneRigidBodyAsset {
        body_type: SceneRigidBodyTypeAsset::Dynamic,
        mass: 3.0,
        mass_properties: Default::default(),
        linear_velocity: [0.0, 0.0, 0.0],
        angular_velocity: [0.0, 0.0, 0.0],
        linear_damping: 0.0,
        angular_damping: 0.0,
        gravity_scale: 1.0,
        ccd_mode: Default::default(),
        sleep_policy: Default::default(),
        lock_translation: [false, false, false],
        lock_rotation: [false, false, false],
    });
    hero_entity.collider = Some(SceneColliderAsset {
        shape: SceneColliderShapeAsset::Sphere { radius: 0.5 },
        sensor: false,
        layer: 1,
        collision_group: 1,
        collision_mask: u32::MAX,
        material: Some(physics_material.clone()),
        material_override: None,
        local_transform: TransformAsset::default(),
    });
    hero_entity.joint = Some(SceneJointAsset {
        joint_type: SceneJointKindAsset::Fixed,
        connected_entity: Some(10),
        anchor: [0.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        limits: None,
        collide_connected: false,
        constraint: Default::default(),
        skeleton_binding: None,
    });
    hero_entity.animation_graph_player = Some(SceneAnimationGraphPlayerAsset {
        graph: animation_graph.clone(),
        parameters: std::collections::BTreeMap::new(),
        playing: true,
    });
    hero_entity.terrain = Some(SceneTerrainAsset {
        terrain: terrain.clone(),
    });
    hero_entity.tilemap = Some(SceneTileMapAsset {
        tilemap: tilemap.clone(),
    });
    hero_entity.prefab_instance = Some(PrefabInstanceAsset {
        prefab: prefab.clone(),
        local_transform: TransformAsset::default(),
        overrides: Vec::new(),
    });

    let scene = SceneAsset {
        entities: vec![camera_entity, hero_entity],
    };

    assert_eq!(
        scene.direct_references(),
        vec![
            camera_target.clone(),
            model.clone(),
            mesh.clone(),
            material.clone(),
            physics_material.clone(),
            animation_graph.clone(),
            terrain.clone(),
            tilemap.clone(),
            prefab.clone(),
        ]
    );
    assert_eq!(
        scene.entities[1].direct_references(),
        vec![
            model,
            mesh,
            material,
            physics_material,
            animation_graph,
            terrain,
            tilemap,
            prefab
        ]
    );

    let overview: SceneAssetOverview = scene.overview();

    assert_eq!(overview.entity_count, 2);
    assert_eq!(overview.active_entity_count, 1);
    assert_eq!(overview.root_entity_count, 1);
    assert_eq!(overview.camera_count, 1);
    assert_eq!(overview.mesh_instance_count, 1);
    assert_eq!(overview.direct_mesh_reference_count, 1);
    assert_eq!(overview.mesh_primitive_binding_count, 0);
    assert_eq!(overview.morph_weight_count, 2);
    assert_eq!(overview.mesh_material_binding_count, 1);
    assert_eq!(overview.collider_material_binding_count, 1);
    assert_eq!(overview.light_count, 1);
    assert_eq!(overview.physics_component_count, 3);
    assert_eq!(overview.animation_binding_count, 1);
    assert_eq!(overview.terrain_count, 1);
    assert_eq!(overview.tilemap_count, 1);
    assert_eq!(overview.prefab_instance_count, 1);
    assert_eq!(overview.direct_reference_count, 9);

    let camera_overview = &overview.entities[0];
    assert_eq!(camera_overview.entity, 10);
    assert_eq!(camera_overview.name, "Camera");
    assert_eq!(camera_overview.direct_reference_count, 1);
    assert!(camera_overview.has_camera);
    assert!(!camera_overview.has_mesh);

    let hero_overview = &overview.entities[1];
    assert_eq!(hero_overview.entity, 11);
    assert_eq!(hero_overview.parent, Some(10));
    assert!(!hero_overview.active);
    assert_eq!(hero_overview.render_layer_mask, 0x0000_0010);
    assert_eq!(hero_overview.mobility, SceneMobilityAsset::Static);
    assert_eq!(hero_overview.direct_reference_count, 8);
    assert!(hero_overview.has_mesh);
    assert!(hero_overview.has_direct_mesh_reference);
    assert_eq!(hero_overview.direct_mesh_reference_count, 1);
    assert_eq!(hero_overview.mesh_primitive_binding_count, 0);
    assert_eq!(hero_overview.morph_weight_count, 2);
    assert!(hero_overview.has_collider_material);
    assert_eq!(hero_overview.light_count(), 1);
    assert_eq!(hero_overview.physics_component_count(), 3);
    assert_eq!(hero_overview.animation_binding_count(), 1);

    let scene_id = ResourceId::from_stable_label("res://scenes/overview.scene.toml");
    let record: SceneAssetManagementRecord = scene.management_record(scene_id);

    assert_eq!(record.scene_id, scene_id);
    assert_eq!(record.overview, overview);
}

#[test]
fn scene_asset_overview_handles_empty_scenes() {
    let scene = SceneAsset {
        entities: Vec::new(),
    };

    let overview = scene.overview();

    assert_eq!(overview.entity_count, 0);
    assert_eq!(overview.direct_reference_count, 0);
    assert!(overview.entities.is_empty());
    assert!(scene.direct_references().is_empty());
}

#[test]
fn scene_asset_management_record_set_sorts_and_summarizes_records() {
    let camera_target = asset_ref(
        "camera-target-record-set",
        "res://textures/camera-target-record-set.png",
    );
    let model = asset_ref("scene-record-set-model", "res://models/record-set.gltf");
    let material = asset_ref(
        "scene-record-set-material",
        "res://materials/record-set.zmaterial",
    );
    let clip = asset_ref(
        "scene-record-set-clip",
        "res://animation/record-set.clip.zranim",
    );

    let mut camera_entity = empty_scene_entity(20, "RecordSetCamera");
    camera_entity.camera = Some(SceneCameraAsset {
        target: SceneCameraTargetAsset::Texture {
            texture: camera_target,
        },
        post_process_settings: None,
        ..SceneCameraAsset::default()
    });

    let mut actor_entity = empty_scene_entity(21, "RecordSetActor");
    actor_entity.parent = Some(20);
    actor_entity.active = false;
    actor_entity.mesh = Some(SceneMeshInstanceAsset {
        model,
        mesh: None,
        material,
        render_queue: 0,
        material_queue: 0,
        order_in_layer: 0,
        depth_bias: 0.0,
        morph_weights: Vec::new(),
        primitives: Vec::new(),
        lods: Vec::new(),
    });
    actor_entity.directional_light = Some(SceneDirectionalLightAsset {
        direction: [0.0, -1.0, 0.0],
        color: [1.0, 1.0, 1.0],
        intensity: 2.0,
        volumetric: false,
    });
    actor_entity.rigid_body = Some(SceneRigidBodyAsset {
        body_type: SceneRigidBodyTypeAsset::Dynamic,
        mass: 1.0,
        mass_properties: Default::default(),
        linear_velocity: [0.0, 0.0, 0.0],
        angular_velocity: [0.0, 0.0, 0.0],
        linear_damping: 0.0,
        angular_damping: 0.0,
        gravity_scale: 1.0,
        ccd_mode: Default::default(),
        sleep_policy: Default::default(),
        lock_translation: [false, false, false],
        lock_rotation: [false, false, false],
    });
    actor_entity.animation_player = Some(SceneAnimationPlayerAsset {
        clip,
        playback_speed: 1.0,
        time_seconds: 0.0,
        weight: 1.0,
        looping: true,
        playing: true,
    });

    let populated_scene = SceneAsset {
        entities: vec![camera_entity, actor_entity],
    };
    let empty_scene = SceneAsset {
        entities: Vec::new(),
    };
    let populated_id = ResourceId::from_stable_label("scene:record-set-populated");
    let empty_id = ResourceId::from_stable_label("scene:record-set-empty");

    let record_set = SceneAssetManagementRecordSet::from_records(vec![
        populated_scene.management_record(populated_id),
        empty_scene.management_record(empty_id),
    ]);

    let mut expected_ids = vec![empty_id, populated_id];
    expected_ids.sort();
    let record_ids = record_set
        .records
        .iter()
        .map(|record| record.scene_id)
        .collect::<Vec<_>>();
    assert_eq!(record_ids, expected_ids);
    assert_eq!(record_set.records.len(), 2);
    let summary = &record_set.summary;
    assert_eq!(summary.scene_count, 2);
    assert_eq!(summary.entity_count, 2);
    assert_eq!(summary.active_entity_count, 1);
    assert_eq!(summary.root_entity_count, 1);
    assert_eq!(summary.direct_reference_count, 4);
    assert_eq!(summary.camera_count, 1);
    assert_eq!(summary.mesh_instance_count, 1);
    assert_eq!(summary.direct_mesh_reference_count, 0);
    assert_eq!(summary.mesh_primitive_binding_count, 0);
    assert_eq!(summary.mesh_material_binding_count, 1);
    assert_eq!(summary.collider_material_binding_count, 0);
    assert_eq!(summary.light_count, 1);
    assert_eq!(summary.physics_component_count, 1);
    assert_eq!(summary.animation_binding_count, 1);
    assert_eq!(summary.terrain_count, 0);
    assert_eq!(summary.tilemap_count, 0);
    assert_eq!(summary.prefab_instance_count, 0);

    let entity_record_set = SceneEntityManagementRecordSet::from_records(
        record_set
            .records
            .iter()
            .flat_map(SceneAssetManagementRecord::entity_management_records)
            .collect(),
    );

    assert_eq!(
        populated_scene
            .entity_management_records(populated_id)
            .iter()
            .map(|record| record.entity.entity)
            .collect::<Vec<_>>(),
        vec![20, 21]
    );
    assert_eq!(
        entity_record_set
            .records
            .iter()
            .map(|record| (record.scene_id, record.entity.entity))
            .collect::<Vec<_>>(),
        vec![(populated_id, 20), (populated_id, 21)]
    );
    assert_eq!(entity_record_set.summary.scene_count, 1);
    assert_eq!(entity_record_set.summary.entity_count, 2);
    assert_eq!(entity_record_set.summary.active_entity_count, 1);
    assert_eq!(entity_record_set.summary.root_entity_count, 1);
    assert_eq!(entity_record_set.summary.direct_reference_count, 4);
    assert_eq!(entity_record_set.summary.camera_count, 1);
    assert_eq!(entity_record_set.summary.mesh_instance_count, 1);
    assert_eq!(entity_record_set.summary.direct_mesh_reference_count, 0);
    assert_eq!(entity_record_set.summary.mesh_primitive_binding_count, 0);
    assert_eq!(entity_record_set.summary.mesh_material_binding_count, 1);
    assert_eq!(entity_record_set.summary.collider_material_binding_count, 0);
    assert_eq!(entity_record_set.summary.light_count, 1);
    assert_eq!(entity_record_set.summary.physics_component_count, 1);
    assert_eq!(entity_record_set.summary.animation_binding_count, 1);
    assert_eq!(entity_record_set.summary.terrain_count, 0);
    assert_eq!(entity_record_set.summary.tilemap_count, 0);
    assert_eq!(entity_record_set.summary.prefab_instance_count, 0);
}

fn empty_scene_entity(entity: u64, name: &str) -> SceneEntityAsset {
    SceneEntityAsset {
        entity,
        name: name.to_string(),
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
    }
}

fn asset_ref(label: &str, uri: &str) -> AssetReference {
    AssetReference::new(
        AssetUuid::from_stable_label(label),
        AssetUri::parse(uri).unwrap(),
    )
}
