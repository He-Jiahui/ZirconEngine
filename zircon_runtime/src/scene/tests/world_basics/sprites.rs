use super::*;

#[test]
fn render_product_sprite_world_frame_extract_exposes_runtime_sprite_components() {
    let mut world = World::empty();
    let camera = world.spawn_node(NodeKind::Camera);
    world
        .insert(
            camera,
            CameraComponent {
                core_pipeline: CorePipelineKind::Core2d,
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
                core_pipeline: CorePipelineKind::Core2d,
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

    assert!(
        extract
            .sprites
            .sprites
            .iter()
            .any(|sprite| sprite.entity == visible_sprite)
    );
    assert!(
        extract
            .sprites
            .sprites
            .iter()
            .all(|sprite| sprite.entity != hidden_sprite)
    );
    assert!(extract.sprites.sprites.iter().all(|sprite| {
        sprite
            .common
            .layer_mask
            .intersects_scene_schema_v1_mask(0b0010)
    }));
    assert!(
        extract
            .visibility
            .dynamic_entities
            .contains(&visible_sprite)
    );
    assert!(!extract.visibility.dynamic_entities.contains(&hidden_sprite));
}

#[test]
fn render_product_sprite_world_frame_extract_projects_static_mobility_into_common() {
    let mut world = World::empty();
    let camera = world.spawn_node(NodeKind::Camera);
    world
        .insert(
            camera,
            CameraComponent {
                core_pipeline: CorePipelineKind::Core2d,
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
                image: texture_handle("res://textures/static.png"),
                ..Sprite2dComponent::default()
            },
        )
        .unwrap();
    world
        .set_mobility(
            sprite_entity,
            crate::core::framework::scene::Mobility::Static,
        )
        .unwrap();

    let extract = world.to_render_frame_extract();
    let sprite = extract
        .sprites
        .sprites
        .iter()
        .find(|sprite| sprite.entity == sprite_entity)
        .expect("static sprite should be extracted");

    assert!(sprite.common.is_static);
    assert!(extract.visibility.static_entities.contains(&sprite_entity));
    assert!(!extract.visibility.dynamic_entities.contains(&sprite_entity));
}

#[test]
fn render_product_sprite_mesh2d_component_does_not_count_as_particle_sprite() {
    let mut world = World::empty();
    let camera = world.spawn_node(NodeKind::Camera);
    world
        .insert(
            camera,
            CameraComponent {
                core_pipeline: CorePipelineKind::Core2d,
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
