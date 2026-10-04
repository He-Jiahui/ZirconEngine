use super::*;

use crate::asset::{SceneMesh2dAsset, SceneSprite2dAsset};
use crate::core::framework::render::{
    CorePipelineKind, ProjectionMode, RenderMaterialAlphaMode, RenderSpriteAnchor,
    RenderSpriteAtlasRegion, RenderSpriteImageMode, RenderSpriteRect, RenderSpriteSliceBorder,
    RenderSpriteSliceScaleMode, RenderSpriteSlicer,
};
use crate::core::math::Vec4;
use crate::core::resource::{ResourceHandle, TextureMarker};
use crate::scene::components::{Mesh2dComponent, Sprite2dComponent};

fn project_texture_handle(
    project: &crate::asset::project::ProjectManager,
    uri: &str,
) -> ResourceHandle<TextureMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    ResourceHandle::new(
        project
            .asset_registry()
            .resolve_asset_id_by_path(&uri)
            .unwrap(),
    )
}

#[test]
fn scene_assets_roundtrip_sprite_and_mesh2d_through_project_reimport_and_render_extract() {
    let root = unique_temp_project_root("scene_2d_project_reopen");
    let mut project = create_test_project(&root);
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let mut world = World::load_scene_from_uri(&project, &scene_uri).unwrap();

    // Older canonical documents have no 2D table and continue to load as absent components.
    assert!(world.nodes().iter().all(|node| {
        world.get::<Sprite2dComponent>(node.id).is_none()
            && world.get::<Mesh2dComponent>(node.id).is_none()
    }));

    let camera = world.active_camera();
    let camera_component = world.get_mut::<CameraComponent>(camera).unwrap();
    camera_component.core_pipeline = CorePipelineKind::Core2d;
    camera_component.projection_mode = ProjectionMode::Orthographic;

    let sprite_entity = world.spawn_node(NodeKind::Empty).unwrap();
    let sprite = Sprite2dComponent {
        image: project_texture_handle(&project, "res://textures/checker.png"),
        material: Some(project_material_handle(
            &project,
            "res://materials/grid.zmaterial",
        )),
        atlas_region: Some(RenderSpriteAtlasRegion {
            min: crate::core::math::Vec2::new(0.25, 0.5),
            max: crate::core::math::Vec2::new(0.5, 0.75),
        }),
        rect: Some(RenderSpriteRect {
            min: crate::core::math::Vec2::new(4.0, 8.0),
            max: crate::core::math::Vec2::new(20.0, 40.0),
        }),
        flip_x: true,
        flip_y: false,
        anchor: RenderSpriteAnchor::TOP_LEFT,
        custom_size: Some(crate::core::math::Vec2::new(2.0, 4.0)),
        image_mode: RenderSpriteImageMode::Sliced(RenderSpriteSlicer {
            border: RenderSpriteSliceBorder::all(4.0),
            center_scale_mode: RenderSpriteSliceScaleMode::Tile { stretch_value: 0.5 },
            sides_scale_mode: RenderSpriteSliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }),
        color: Vec4::new(0.5, 0.75, 1.0, 0.6),
        z_order: 3,
        material_alpha_mode: RenderMaterialAlphaMode::Blend,
    };
    world.insert(sprite_entity, sprite.clone()).unwrap();

    let mesh_entity = world.spawn_node(NodeKind::Empty).unwrap();
    let mesh_2d = Mesh2dComponent {
        mesh: project_model_handle(&project, "res://models/triangle.obj"),
        material: project_material_handle(&project, "res://materials/grid.zmaterial"),
        color: Vec4::new(1.0, 0.25, 0.5, 1.0),
        z_order: -2,
        material_alpha_mode: RenderMaterialAlphaMode::Mask { cutoff: 0.37 },
    };
    world.insert(mesh_entity, mesh_2d.clone()).unwrap();

    let expected_scene = world.to_scene_asset(&project).unwrap();
    let saved_sprite_entity = expected_scene
        .entities
        .iter()
        .find(|entity| entity.entity == sprite_entity)
        .unwrap();
    assert_eq!(saved_sprite_entity.direct_reference_count(), 2);
    assert_eq!(
        saved_sprite_entity
            .direct_references()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec![
            "res://textures/checker.png",
            "res://materials/grid.zmaterial"
        ]
    );
    let saved_mesh_entity = expected_scene
        .entities
        .iter()
        .find(|entity| entity.entity == mesh_entity)
        .unwrap();
    assert_eq!(saved_mesh_entity.direct_reference_count(), 2);
    assert_eq!(
        saved_mesh_entity
            .direct_references()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec![
            "res://models/triangle.obj",
            "res://materials/grid.zmaterial"
        ]
    );

    world.save_scene_to_project(&project, &scene_uri).unwrap();
    let source_path = project
        .existing_or_primary_project_source_path_for_uri(&scene_uri)
        .unwrap();
    let first_save = std::fs::read_to_string(&source_path).unwrap();
    assert!(first_save.contains("zircon.render2d.sprite"));
    assert!(first_save.contains("zircon.render2d.mesh"));
    project.scan_and_import().unwrap();

    let scene_record = project
        .asset_registry()
        .entry_by_path(&scene_uri)
        .expect("saved scene should be indexed after reimport");
    for uri in [
        "res://textures/checker.png",
        "res://models/triangle.obj",
        "res://materials/grid.zmaterial",
    ] {
        let dependency_uri = AssetUri::parse(uri).unwrap();
        let dependency = project
            .asset_registry()
            .entry_by_path(&dependency_uri)
            .expect("2D project dependency should be registered")
            .uuid();
        assert!(scene_record.dependencies().contains(&dependency));
    }
    drop(project);

    let mut reopened_project = crate::asset::project::ProjectManager::open(&root).unwrap();
    reopened_project
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    reopened_project.scan_and_import().unwrap();
    let ImportedAsset::Scene(reopened_asset) = reopened_project.load_artifact(&scene_uri).unwrap()
    else {
        panic!("saved project artifact should remain a scene");
    };
    assert_eq!(reopened_asset, expected_scene);

    let reopened_world = World::load_scene_from_uri(&reopened_project, &scene_uri).unwrap();
    assert_eq!(
        reopened_world.get::<Sprite2dComponent>(sprite_entity),
        Some(&sprite)
    );
    assert_eq!(
        reopened_world.get::<Mesh2dComponent>(mesh_entity),
        Some(&mesh_2d)
    );
    let extract = reopened_world.to_render_frame_extract();
    let extracted_sprite = extract
        .sprites
        .sprites
        .iter()
        .find(|sprite| sprite.entity == sprite_entity)
        .expect("reopened 2D sprite should reach the runtime render extract");
    assert_eq!(extracted_sprite.image, sprite.image);
    assert_eq!(extracted_sprite.material, sprite.material);
    assert_eq!(extracted_sprite.atlas_region, sprite.atlas_region);
    assert_eq!(extracted_sprite.rect, sprite.rect);
    assert_eq!(extracted_sprite.image_mode, sprite.image_mode);
    assert_eq!(extracted_sprite.color, sprite.color);
    assert_eq!(
        extracted_sprite.material_alpha_mode,
        sprite.material_alpha_mode
    );

    drop(reopened_project);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn scene_asset_roundtrips_builtin_2d_component_default_resources() {
    let root = unique_temp_project_root("scene_2d_builtin_defaults");
    let project = create_test_project(&root);
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let sprite = Sprite2dComponent::default();
    let mesh_2d = Mesh2dComponent::default();
    world.insert(entity, sprite.clone()).unwrap();
    world.insert(entity, mesh_2d.clone()).unwrap();

    let saved = world.to_scene_asset(&project).unwrap();
    let sprite_row = saved.entities[0]
        .components
        .iter()
        .find(|row| row.type_id == "zircon.render2d.sprite")
        .unwrap();
    assert_eq!(
        sprite_row
            .decode_typed::<SceneSprite2dAsset>()
            .unwrap()
            .image
            .to_string(),
        "builtin://missing-texture"
    );
    let mesh_row = saved.entities[0]
        .components
        .iter()
        .find(|row| row.type_id == "zircon.render2d.mesh")
        .unwrap();
    assert_eq!(
        mesh_row
            .decode_typed::<SceneMesh2dAsset>()
            .unwrap()
            .model
            .to_string(),
        "builtin://quad"
    );
    let reopened = World::from_scene_asset(&project, &saved).unwrap();
    assert_eq!(reopened.get::<Sprite2dComponent>(entity), Some(&sprite));
    assert_eq!(reopened.get::<Mesh2dComponent>(entity), Some(&mesh_2d));

    drop(project);
    let _ = std::fs::remove_dir_all(root);
}
