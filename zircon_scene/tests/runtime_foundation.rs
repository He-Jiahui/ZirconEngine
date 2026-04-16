use zircon_math::{Transform, Vec3};
use zircon_scene::{Mobility, NodeKind, World};

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_asset::{
    AssetUri, ProjectManager, ProjectManifest, ProjectPaths, SceneAsset, SceneCameraAsset,
    SceneEntityAsset, SceneMobilityAsset, TransformAsset,
};

#[test]
fn spawned_entities_start_with_runtime_foundation_defaults() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Mesh);

    assert_eq!(world.active_self(entity), Some(true));
    assert_eq!(world.active_in_hierarchy(entity), Some(true));
    assert_eq!(world.render_layer_mask(entity), Some(0x0000_0001));
    assert_eq!(world.mobility(entity), Some(Mobility::Dynamic));
    assert_eq!(
        world
            .world_matrix(entity)
            .unwrap()
            .transform_point3(Vec3::ZERO),
        Vec3::ZERO
    );
}

#[test]
fn active_self_and_active_in_hierarchy_are_distinct_and_propagated() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(parent)).unwrap();

    world.set_active_self(parent, false).unwrap();

    assert_eq!(world.active_self(parent), Some(false));
    assert_eq!(world.active_self(child), Some(true));
    assert_eq!(world.active_in_hierarchy(parent), Some(false));
    assert_eq!(world.active_in_hierarchy(child), Some(false));

    world.set_active_self(parent, true).unwrap();

    assert_eq!(world.active_in_hierarchy(parent), Some(true));
    assert_eq!(world.active_in_hierarchy(child), Some(true));
}

#[test]
fn world_matrix_rebuilds_after_transform_updates_and_reparenting() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);

    world
        .update_transform(
            parent,
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();

    assert_eq!(
        world
            .world_matrix(child)
            .unwrap()
            .transform_point3(Vec3::ZERO),
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert_eq!(
        world.find_node(child).unwrap().transform.translation,
        Vec3::new(2.0, 0.0, 0.0)
    );
}

#[test]
fn static_entities_reject_runtime_transform_updates_and_dynamic_parenting() {
    let mut world = World::new();
    let static_entity = world.spawn_node(NodeKind::Mesh);
    let dynamic_parent = world.spawn_node(NodeKind::Cube);

    world.set_mobility(static_entity, Mobility::Static).unwrap();

    let error = world
        .update_transform(
            static_entity,
            Transform::from_translation(Vec3::new(3.0, 1.0, 0.0)),
        )
        .unwrap_err();
    assert!(error.contains("Static"));
    assert_eq!(
        world
            .world_matrix(static_entity)
            .unwrap()
            .transform_point3(Vec3::ZERO),
        Vec3::ZERO
    );

    let error = world
        .set_parent_checked(static_entity, Some(dynamic_parent))
        .unwrap_err();
    assert!(error.contains("Static"));
}

#[test]
fn render_layer_mask_and_mobility_survive_world_roundtrip() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Mesh);
    let path = std::env::temp_dir().join(format!(
        "zircon_scene_runtime_foundation_{}.json",
        std::process::id()
    ));

    world.set_render_layer_mask(entity, 0x0000_0010).unwrap();
    world.set_mobility(entity, Mobility::Static).unwrap();
    world.save_project_to_path(&path).unwrap();

    let loaded = World::load_project_from_path(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    assert_eq!(loaded.render_layer_mask(entity), Some(0x0000_0010));
    assert_eq!(loaded.mobility(entity), Some(Mobility::Static));
    assert_eq!(loaded.active_self(entity), Some(true));
    assert_eq!(loaded.active_in_hierarchy(entity), Some(true));
}

#[test]
fn legacy_scene_assets_default_runtime_foundation_fields_on_world_load() {
    let root = unique_temp_project_root("legacy_scene_defaults");
    let project = create_empty_project(&root);
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 7,
            name: "Camera".to_string(),
            parent: None,
            transform: TransformAsset {
                translation: [0.0, 2.0, 5.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            },
            active: true,
            render_layer_mask: 0x0000_0001,
            mobility: SceneMobilityAsset::Dynamic,
            camera: Some(SceneCameraAsset {
                fov_y_radians: 1.0471976,
                z_near: 0.1,
                z_far: 200.0,
            }),
            mesh: None,
            directional_light: None,
        }],
    };
    let document = scene.to_toml_string().unwrap();
    let legacy_document = document
        .lines()
        .filter(|line| !line.contains("render_layer_mask") && !line.contains("mobility"))
        .collect::<Vec<_>>()
        .join("\n");
    let legacy_scene = SceneAsset::from_toml_str(&legacy_document).unwrap();

    let world = World::from_scene_asset(&project, &legacy_scene).unwrap();

    assert_eq!(world.render_layer_mask(7), Some(0x0000_0001));
    assert_eq!(world.mobility(7), Some(Mobility::Dynamic));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scene_asset_roundtrip_preserves_render_layer_mask_and_mobility() {
    let root = unique_temp_project_root("scene_asset_roundtrip_runtime_foundation");
    let project = create_empty_project(&root);
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 11,
            name: "Camera".to_string(),
            parent: None,
            transform: TransformAsset {
                translation: [0.0, 2.0, 5.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            },
            active: true,
            render_layer_mask: 0x0000_0010,
            mobility: SceneMobilityAsset::Static,
            camera: Some(SceneCameraAsset {
                fov_y_radians: 1.0471976,
                z_near: 0.1,
                z_far: 200.0,
            }),
            mesh: None,
            directional_light: None,
        }],
    };

    let world = World::from_scene_asset(&project, &scene).unwrap();
    let roundtripped = world.to_scene_asset(&project).unwrap();
    let entity = &roundtripped.entities[0];

    assert_eq!(entity.render_layer_mask, 0x0000_0010);
    assert_eq!(entity.mobility, SceneMobilityAsset::Static);

    let _ = fs::remove_dir_all(root);
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_scene_runtime_foundation_{label}_{unique}"))
}

fn create_empty_project(root: &PathBuf) -> ProjectManager {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "RuntimeFoundationSceneAssets",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    ProjectManager::open(root).unwrap()
}
