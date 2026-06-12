use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::framework::render::{
    ProjectionMode, RenderAmbientLightSnapshot, RenderMaterialAlphaMode, RenderPhase,
    RenderPhaseMeshSource, RenderSpriteAnchor, RenderSpriteAtlasRegion, RenderSpriteImageMode,
    RenderSpriteRect, RenderSpriteSliceBorder, RenderSpriteSliceScaleMode, RenderSpriteSlicer,
};
use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::{Transform, Vec2, Vec3, Vec4};

use crate::scene::components::{
    CameraComponent, Mesh2dComponent, MeshRenderer, Name, Sprite2dComponent,
};
use crate::scene::{world::World, NodeKind, SystemStage};

use super::authoring_boundary::{
    assert_text_excludes_authoring_tokens, SERIALIZED_AUTHORING_TOKENS,
};
use super::support::{material_handle, model_handle};

#[test]
fn world_bootstraps_with_renderable_defaults() {
    let world = World::new();
    let snapshot = world.to_render_snapshot();

    assert!(!snapshot.scene.meshes.is_empty());
    assert!(snapshot.overlays.grid.is_none());
    assert!(snapshot.overlays.selection.is_empty());
    assert!(snapshot.overlays.selection_anchors.is_empty());
    assert!(snapshot.overlays.handles.is_empty());
    assert!(snapshot.overlays.scene_gizmos.is_empty());
    assert_eq!(
        world.schedule().stages(),
        vec![
            SystemStage::First,
            SystemStage::PreUpdate,
            SystemStage::FixedUpdate,
            SystemStage::Update,
            SystemStage::PostUpdate,
            SystemStage::Last,
            SystemStage::RenderExtract,
        ]
    );
}

#[test]
fn spawned_entities_have_unique_ids() {
    let mut world = World::new();
    let first = world.spawn_node(NodeKind::Cube);
    let second = world.spawn_node(NodeKind::Cube);
    assert_ne!(first, second);
}

#[test]
fn spawn_node_assigns_one_based_kind_ordinals() {
    let mut world = World::empty();
    let first_mesh = world.spawn_node(NodeKind::Mesh);
    let second_mesh = world.spawn_node(NodeKind::Mesh);
    let first_cube = world.spawn_node(NodeKind::Cube);

    assert_eq!(world.get::<Name>(first_mesh).unwrap().0, "Mesh 1");
    assert_eq!(world.get::<Name>(second_mesh).unwrap().0, "Mesh 2");
    assert_eq!(world.get::<Name>(first_cube).unwrap().0, "Cube 1");
}

#[test]
fn hierarchy_updates_world_transform() {
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
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
}

#[test]
fn updated_transform_is_reflected_in_render_extract() {
    let mut world = World::new();
    let cube = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Cube))
        .unwrap()
        .id;
    world
        .update_transform(cube, Transform::from_translation(Vec3::new(2.0, 3.0, 4.0)))
        .unwrap();

    let snapshot = world.to_render_extract();
    let mesh_snapshot = snapshot
        .scene
        .meshes
        .iter()
        .find(|mesh_snapshot| mesh_snapshot.node_id == cube)
        .unwrap();
    assert_eq!(
        mesh_snapshot.transform.translation,
        Vec3::new(2.0, 3.0, 4.0)
    );
}

#[test]
fn mesh_renderer_sort_fields_feed_geometry_phase_queue() {
    let mut world = World::new();
    let render_first = world.spawn_node(NodeKind::Mesh);
    let depth_earlier = world.spawn_node(NodeKind::Mesh);
    let depth_later = world.spawn_node(NodeKind::Mesh);
    let order_middle = world.spawn_node(NodeKind::Mesh);
    let material_later = world.spawn_node(NodeKind::Mesh);

    let fixtures = [
        (render_first, -10, 90, 90, 0.0),
        (depth_earlier, 0, 0, 0, 2.0),
        (depth_later, 0, 0, 0, 0.0),
        (order_middle, 0, 0, 10, 0.0),
        (material_later, 0, 5, -99, 0.0),
    ];

    for (entity, render_queue, material_queue, order_in_layer, depth_bias) in fixtures {
        let mesh = world.get_mut::<MeshRenderer>(entity).unwrap();
        mesh.material_alpha_mode = RenderMaterialAlphaMode::Blend;
        mesh.render_queue = render_queue;
        mesh.material_queue = material_queue;
        mesh.order_in_layer = order_in_layer;
        mesh.depth_bias = depth_bias;
        world
            .update_transform(
                entity,
                Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
            )
            .unwrap();
    }

    let extract = world.to_render_frame_extract();
    let mesh_index = |entity| {
        extract
            .geometry
            .meshes
            .iter()
            .position(|mesh| mesh.node_id == entity)
            .expect("mesh should be extracted")
    };
    let render_first_index = mesh_index(render_first);
    let depth_earlier_index = mesh_index(depth_earlier);
    let depth_later_index = mesh_index(depth_later);
    let order_middle_index = mesh_index(order_middle);
    let material_later_index = mesh_index(material_later);

    for (entity, render_queue, material_queue, order_in_layer, depth_bias) in fixtures {
        let mesh_index = mesh_index(entity);
        let input = extract
            .geometry
            .phase_inputs
            .iter()
            .find(|input| input.entity == entity && input.mesh_index == mesh_index)
            .expect("phase input should carry mesh sort fields");
        assert_eq!(input.render_queue, render_queue);
        assert_eq!(input.material_queue, material_queue);
        assert_eq!(input.order_in_layer, order_in_layer);
        assert_eq!(input.depth_bias, depth_bias);
    }

    assert_eq!(
        extract
            .geometry
            .phase_queue
            .items_for_phase(RenderPhase::Transparent3d)
            .filter(|item| {
                [
                    render_first,
                    depth_earlier,
                    depth_later,
                    order_middle,
                    material_later,
                ]
                .contains(&item.entity)
            })
            .map(|item| item.mesh_source)
            .collect::<Vec<_>>(),
        vec![
            RenderPhaseMeshSource::MeshIndex(render_first_index),
            RenderPhaseMeshSource::MeshIndex(depth_earlier_index),
            RenderPhaseMeshSource::MeshIndex(depth_later_index),
            RenderPhaseMeshSource::MeshIndex(order_middle_index),
            RenderPhaseMeshSource::MeshIndex(material_later_index),
        ]
    );
}

#[test]
fn project_roundtrip_preserves_imported_meshes() {
    let mut world = World::new();
    let imported = world.spawn_mesh_node(
        model_handle("res://models/robot.obj"),
        material_handle("res://materials/robot.zmaterial"),
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_scene_roundtrip_{unique}.json"));
    world.save_project_to_path(&path).unwrap();
    let saved = fs::read_to_string(&path).unwrap();
    let loaded = World::load_project_from_path(&path).unwrap();
    let _ = fs::remove_file(&path);

    assert_text_excludes_authoring_tokens(
        "world project serialization",
        &saved,
        SERIALIZED_AUTHORING_TOKENS,
    );
    let imported_node = loaded.find_node(imported).unwrap();
    assert!(matches!(imported_node.kind, NodeKind::Mesh));
    assert_eq!(
        imported_node.mesh.as_ref().unwrap().model,
        model_handle("res://models/robot.obj")
    );
}

#[test]
fn node_record_roundtrip_restores_same_entity() {
    let mut world = World::new();
    let cube = world.spawn_node(NodeKind::Cube);
    let record = world.node_record(cube).unwrap();

    assert!(world.remove_entity(cube));
    assert!(!world.contains_entity(cube));

    world.insert_node_record(record.clone()).unwrap();

    let restored = world.node_record(cube).unwrap();
    assert_eq!(restored, record);
}

#[test]
fn recursive_remove_returns_parent_and_children_records() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(parent)).unwrap();

    let removed = world.remove_entity_recursive(parent);
    assert_eq!(removed.len(), 2);
    assert!(!world.contains_entity(parent));
    assert!(!world.contains_entity(child));
}

#[test]
fn set_parent_checked_rejects_hierarchy_cycles() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(parent)).unwrap();

    let error = world.set_parent_checked(parent, Some(child)).unwrap_err();

    assert!(error.contains("cycle"));
    assert_eq!(world.find_node(parent).unwrap().parent, None);
    assert_eq!(world.find_node(child).unwrap().parent, Some(parent));
}

#[test]
fn render_extract_separates_directional_point_and_spot_lights() {
    let mut world = World::new();
    let point = world.spawn_node(NodeKind::PointLight);
    let spot = world.spawn_node(NodeKind::SpotLight);

    world
        .update_transform(point, Transform::from_translation(Vec3::new(3.0, 4.0, 5.0)))
        .unwrap();
    world
        .update_transform(spot, Transform::from_translation(Vec3::new(-2.0, 6.0, 1.5)))
        .unwrap();

    world
        .set_property(
            point,
            &ComponentPropertyPath::parse("PointLight.color").unwrap(),
            ScenePropertyValue::Vec3([0.2, 0.4, 0.8]),
        )
        .unwrap();
    world
        .set_property(
            point,
            &ComponentPropertyPath::parse("PointLight.intensity").unwrap(),
            ScenePropertyValue::Scalar(6.5),
        )
        .unwrap();
    world
        .set_property(
            point,
            &ComponentPropertyPath::parse("PointLight.range").unwrap(),
            ScenePropertyValue::Scalar(9.0),
        )
        .unwrap();

    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.direction").unwrap(),
            ScenePropertyValue::Vec3([0.0, -1.0, 0.25]),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.color").unwrap(),
            ScenePropertyValue::Vec3([1.0, 0.8, 0.3]),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.intensity").unwrap(),
            ScenePropertyValue::Scalar(12.0),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.range").unwrap(),
            ScenePropertyValue::Scalar(15.0),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.inner_angle_radians").unwrap(),
            ScenePropertyValue::Scalar(0.35),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.outer_angle_radians").unwrap(),
            ScenePropertyValue::Scalar(0.65),
        )
        .unwrap();

    let snapshot = world.to_render_extract();

    assert_eq!(snapshot.scene.directional_lights.len(), 1);

    let point_light = snapshot
        .scene
        .point_lights
        .iter()
        .find(|light| light.node_id == point)
        .unwrap();
    assert_eq!(point_light.position, Vec3::new(3.0, 4.0, 5.0));
    assert_eq!(point_light.color, Vec3::new(0.2, 0.4, 0.8));
    assert_eq!(point_light.intensity, 6.5);
    assert_eq!(point_light.range, 9.0);

    let spot_light = snapshot
        .scene
        .spot_lights
        .iter()
        .find(|light| light.node_id == spot)
        .unwrap();
    assert_eq!(spot_light.position, Vec3::new(-2.0, 6.0, 1.5));
    assert_eq!(spot_light.direction, Vec3::new(0.0, -1.0, 0.25));
    assert_eq!(spot_light.color, Vec3::new(1.0, 0.8, 0.3));
    assert_eq!(spot_light.intensity, 12.0);
    assert_eq!(spot_light.range, 15.0);
    assert_eq!(spot_light.inner_angle_radians, 0.35);
    assert_eq!(spot_light.outer_angle_radians, 0.65);

    let frame_extract = world.to_render_frame_extract();
    assert_eq!(frame_extract.lighting.directional_lights.len(), 1);
    assert!(frame_extract
        .lighting
        .point_lights
        .iter()
        .any(|light| light.node_id == point));
    assert!(frame_extract
        .lighting
        .spot_lights
        .iter()
        .any(|light| light.node_id == spot));
}

#[test]
fn render_product_pbr_world_frame_extract_exposes_authored_ambient_and_rect_light_slots() {
    let mut world = World::new();
    let ambient = world.spawn_node(NodeKind::AmbientLight);
    let rect = world.spawn_node(NodeKind::RectLight);

    world
        .update_transform(
            rect,
            Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)).with_rotation(
                crate::core::math::Quat::from_rotation_y(45.0_f32.to_radians()),
            ),
        )
        .unwrap();
    world
        .set_property(
            ambient,
            &ComponentPropertyPath::parse("AmbientLight.color").unwrap(),
            ScenePropertyValue::Vec3([0.05, 0.06, 0.07]),
        )
        .unwrap();
    world
        .set_property(
            ambient,
            &ComponentPropertyPath::parse("AmbientLight.intensity").unwrap(),
            ScenePropertyValue::Scalar(0.35),
        )
        .unwrap();
    world
        .set_property(
            ambient,
            &ComponentPropertyPath::parse("AmbientLight.affects_lightmapped_meshes").unwrap(),
            ScenePropertyValue::Bool(false),
        )
        .unwrap();
    world
        .set_property(
            rect,
            &ComponentPropertyPath::parse("RectLight.color").unwrap(),
            ScenePropertyValue::Vec3([1.0, 0.8, 0.6]),
        )
        .unwrap();
    world
        .set_property(
            rect,
            &ComponentPropertyPath::parse("RectLight.intensity").unwrap(),
            ScenePropertyValue::Scalar(12.0),
        )
        .unwrap();
    world
        .set_property(
            rect,
            &ComponentPropertyPath::parse("RectLight.range").unwrap(),
            ScenePropertyValue::Scalar(16.0),
        )
        .unwrap();
    world
        .set_property(
            rect,
            &ComponentPropertyPath::parse("RectLight.size").unwrap(),
            ScenePropertyValue::Vec2([2.0, 0.5]),
        )
        .unwrap();

    let snapshot = world.to_render_extract();
    assert_eq!(snapshot.scene.ambient_lights.len(), 1);
    assert_eq!(
        snapshot.scene.ambient_lights[0].color,
        Vec3::new(0.05, 0.06, 0.07)
    );
    assert_eq!(snapshot.scene.ambient_lights[0].intensity, 0.35);
    assert!(!snapshot.scene.ambient_lights[0].renderer_degraded);
    assert_eq!(snapshot.scene.ambient_lights[0].degradation_reason, None);

    let rect_light = snapshot
        .scene
        .rect_lights
        .iter()
        .find(|light| light.node_id == rect)
        .unwrap();
    assert_eq!(rect_light.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(
        rect_light.direction,
        world.world_transform(rect).unwrap().forward()
    );
    assert_eq!(rect_light.color, Vec3::new(1.0, 0.8, 0.6));
    assert_eq!(rect_light.intensity, 12.0);
    assert_eq!(rect_light.range, 16.0);
    assert_eq!(rect_light.size, Vec2::new(2.0, 0.5));
    assert!(rect_light.renderer_degraded);

    let extract = world.to_render_frame_extract();
    assert_eq!(
        extract.lighting.ambient_lights,
        snapshot.scene.ambient_lights
    );
    assert_eq!(extract.lighting.rect_lights, snapshot.scene.rect_lights);

    let default_ambient = RenderAmbientLightSnapshot::default();
    assert!(default_ambient.renderer_degraded);
    assert!(default_ambient
        .degradation_reason
        .as_deref()
        .unwrap()
        .contains("no authored scene component"));
}

#[test]
fn render_product_sprite_world_frame_extract_exposes_runtime_sprite_components() {
    let mut world = World::empty();
    let camera = world.spawn_node(NodeKind::Camera);
    world
        .insert(
            camera,
            CameraComponent {
                projection_mode: ProjectionMode::Orthographic,
                ..CameraComponent::default()
            },
        )
        .unwrap();
    let sprite_entity = world.spawn_node(NodeKind::Mesh);
    world
        .remove::<crate::scene::components::MeshRenderer>(sprite_entity)
        .unwrap();
    world
        .insert(
            sprite_entity,
            Sprite2dComponent {
                image: texture_handle("res://textures/hero.png"),
                material: Some(material_handle("res://materials/sprite.zmaterial")),
                atlas_region: Some(RenderSpriteAtlasRegion {
                    min: Vec2::new(0.25, 0.5),
                    max: Vec2::new(0.5, 0.75),
                }),
                rect: Some(RenderSpriteRect {
                    min: Vec2::new(4.0, 8.0),
                    max: Vec2::new(20.0, 40.0),
                }),
                flip_x: true,
                flip_y: false,
                anchor: RenderSpriteAnchor::TOP_LEFT,
                custom_size: Some(Vec2::new(2.0, 4.0)),
                image_mode: RenderSpriteImageMode::Sliced(RenderSpriteSlicer {
                    border: RenderSpriteSliceBorder {
                        left: 4.0,
                        right: 6.0,
                        top: 8.0,
                        bottom: 10.0,
                    },
                    center_scale_mode: RenderSpriteSliceScaleMode::Tile { stretch_value: 0.5 },
                    sides_scale_mode: RenderSpriteSliceScaleMode::Stretch,
                    max_corner_scale: 1.0,
                }),
                color: Vec4::new(0.5, 0.75, 1.0, 0.6),
                z_order: 3,
                material_alpha_mode: RenderMaterialAlphaMode::Blend,
            },
        )
        .unwrap();
    world
        .update_transform(
            sprite_entity,
            Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
        )
        .unwrap();

    let extract = world.to_render_frame_extract();

    assert_eq!(
        extract.view.core_pipeline,
        crate::core::framework::render::CorePipelineKind::Core2d
    );
    assert_eq!(extract.sprites.sprites.len(), 1);
    assert!(extract.particles.sprites.is_empty());
    let sprite = &extract.sprites.sprites[0];
    assert_eq!(sprite.entity, sprite_entity);
    assert_eq!(sprite.transform.translation, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(sprite.image, texture_handle("res://textures/hero.png"));
    assert_eq!(
        sprite.material,
        Some(material_handle("res://materials/sprite.zmaterial"))
    );
    assert_eq!(sprite.anchor, RenderSpriteAnchor::TOP_LEFT);
    assert_eq!(sprite.custom_size, Some(Vec2::new(2.0, 4.0)));
    assert_eq!(
        sprite.image_mode,
        RenderSpriteImageMode::Sliced(RenderSpriteSlicer {
            border: RenderSpriteSliceBorder {
                left: 4.0,
                right: 6.0,
                top: 8.0,
                bottom: 10.0,
            },
            center_scale_mode: RenderSpriteSliceScaleMode::Tile { stretch_value: 0.5 },
            sides_scale_mode: RenderSpriteSliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        })
    );
    assert!(sprite.flip_x);
    assert_eq!(sprite.z_order, 3);
    assert_eq!(
        extract
            .sprites
            .phase_queue
            .items_for_phase(RenderPhase::Transparent2d)
            .map(|item| item.mesh_source)
            .collect::<Vec<_>>(),
        vec![RenderPhaseMeshSource::SpriteIndex(0)]
    );
}

#[test]
fn render_product_sprite_world_frame_extract_filters_by_camera_layers() {
    let mut world = World::empty();
    let camera = world.spawn_node(NodeKind::Camera);
    world
        .insert(
            camera,
            CameraComponent {
                projection_mode: ProjectionMode::Orthographic,
                ..CameraComponent::default()
            },
        )
        .unwrap();
    world.set_render_layer_mask(camera, 0b0010).unwrap();

    let visible_sprite = world.spawn_node(NodeKind::Mesh);
    let hidden_sprite = world.spawn_node(NodeKind::Mesh);
    world
        .remove::<crate::scene::components::MeshRenderer>(visible_sprite)
        .unwrap();
    world
        .remove::<crate::scene::components::MeshRenderer>(hidden_sprite)
        .unwrap();
    world.set_render_layer_mask(visible_sprite, 0b0010).unwrap();
    world.set_render_layer_mask(hidden_sprite, 0b0100).unwrap();
    world
        .insert(
            visible_sprite,
            Sprite2dComponent {
                image: texture_handle("res://textures/visible.png"),
                ..Sprite2dComponent::default()
            },
        )
        .unwrap();
    world
        .insert(
            hidden_sprite,
            Sprite2dComponent {
                image: texture_handle("res://textures/hidden.png"),
                ..Sprite2dComponent::default()
            },
        )
        .unwrap();

    let extract = world.to_render_frame_extract();

    assert!(extract
        .sprites
        .sprites
        .iter()
        .any(|sprite| sprite.entity == visible_sprite));
    assert!(extract
        .sprites
        .sprites
        .iter()
        .all(|sprite| sprite.entity != hidden_sprite));
    assert!(extract
        .sprites
        .sprites
        .iter()
        .all(|sprite| sprite.render_layer_mask & 0b0010 != 0));
    assert!(extract
        .visibility
        .dynamic_entities
        .contains(&visible_sprite));
    assert!(!extract.visibility.dynamic_entities.contains(&hidden_sprite));
}

#[test]
fn render_product_sprite_mesh2d_component_does_not_count_as_particle_sprite() {
    let mut world = World::empty();
    let camera = world.spawn_node(NodeKind::Camera);
    world
        .insert(
            camera,
            CameraComponent {
                projection_mode: ProjectionMode::Orthographic,
                ..CameraComponent::default()
            },
        )
        .unwrap();
    let mesh2d_entity = world.spawn_node(NodeKind::Mesh);
    world
        .insert(
            mesh2d_entity,
            Mesh2dComponent {
                mesh: model_handle("res://models/quad.obj"),
                material: material_handle("res://materials/mesh2d.zmaterial"),
                color: Vec4::new(1.0, 0.25, 0.5, 1.0),
                z_order: 5,
                material_alpha_mode: RenderMaterialAlphaMode::Opaque,
            },
        )
        .unwrap();

    let extract = world.to_render_frame_extract();

    assert!(extract.particles.sprites.is_empty());
    assert!(extract.sprites.sprites.is_empty());
}

fn texture_handle(
    label: &str,
) -> crate::core::resource::ResourceHandle<crate::core::resource::TextureMarker> {
    crate::core::resource::ResourceHandle::new(
        crate::core::resource::ResourceId::from_stable_label(label),
    )
}
