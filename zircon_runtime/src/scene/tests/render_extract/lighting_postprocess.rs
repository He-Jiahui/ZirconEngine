use super::*;

#[test]
fn render_frame_extract_filters_lights_by_camera_layers() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world.set_active_camera(camera);

    let visible_ambient = spawn_light_on_layer(&mut world, NodeKind::AmbientLight, 0b0010);
    let hidden_ambient = spawn_light_on_layer(&mut world, NodeKind::AmbientLight, 0b0100);
    let visible_ambient_color = Vec3::new(0.2, 0.3, 0.4);
    let hidden_ambient_color = Vec3::new(0.9, 0.1, 0.1);
    world
        .get_mut::<AmbientLight>(visible_ambient)
        .unwrap()
        .color = visible_ambient_color;
    world
        .get_mut::<AmbientLight>(visible_ambient)
        .unwrap()
        .intensity = 1.5;
    world.get_mut::<AmbientLight>(hidden_ambient).unwrap().color = hidden_ambient_color;
    world
        .get_mut::<AmbientLight>(hidden_ambient)
        .unwrap()
        .intensity = 3.0;

    let visible_directional = spawn_light_on_layer(&mut world, NodeKind::DirectionalLight, 0b0010);
    let hidden_directional = spawn_light_on_layer(&mut world, NodeKind::DirectionalLight, 0b0100);
    let visible_point = spawn_light_on_layer(&mut world, NodeKind::PointLight, 0b0010);
    let hidden_point = spawn_light_on_layer(&mut world, NodeKind::PointLight, 0b0100);
    let visible_rect = spawn_light_on_layer(&mut world, NodeKind::RectLight, 0b0010);
    let hidden_rect = spawn_light_on_layer(&mut world, NodeKind::RectLight, 0b0100);
    let visible_spot = spawn_light_on_layer(&mut world, NodeKind::SpotLight, 0b0010);
    let hidden_spot = spawn_light_on_layer(&mut world, NodeKind::SpotLight, 0b0100);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(704),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.lighting.ambient_lights.len(), 1);
    assert_eq!(
        extract.lighting.ambient_lights[0].color,
        visible_ambient_color
    );
    assert_eq!(extract.lighting.ambient_lights[0].intensity, 1.5);
    assert_ne!(
        extract.lighting.ambient_lights[0].color,
        hidden_ambient_color
    );
    assert!(
        extract
            .lighting
            .directional_lights
            .iter()
            .any(|light| light.node_id == visible_directional)
    );
    assert!(
        extract
            .lighting
            .directional_lights
            .iter()
            .all(|light| light.node_id != hidden_directional)
    );
    assert!(
        extract
            .lighting
            .point_lights
            .iter()
            .any(|light| light.node_id == visible_point)
    );
    assert!(
        extract
            .lighting
            .point_lights
            .iter()
            .all(|light| light.node_id != hidden_point)
    );
    assert!(
        extract
            .lighting
            .rect_lights
            .iter()
            .any(|light| light.node_id == visible_rect)
    );
    assert!(
        extract
            .lighting
            .rect_lights
            .iter()
            .all(|light| light.node_id != hidden_rect)
    );
    assert!(
        extract
            .lighting
            .spot_lights
            .iter()
            .any(|light| light.node_id == visible_spot)
    );
    assert!(
        extract
            .lighting
            .spot_lights
            .iter()
            .all(|light| light.node_id != hidden_spot)
    );

    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest::default());
    assert_eq!(packet.scene.ambient_lights.len(), 1);
    assert_eq!(packet.scene.ambient_lights[0].color, visible_ambient_color);
    assert!(
        packet
            .scene
            .directional_lights
            .iter()
            .any(|light| light.node_id == visible_directional)
    );
    assert!(
        packet
            .scene
            .directional_lights
            .iter()
            .all(|light| light.node_id != hidden_directional)
    );
    assert!(
        packet
            .scene
            .point_lights
            .iter()
            .any(|light| light.node_id == visible_point)
    );
    assert!(
        packet
            .scene
            .point_lights
            .iter()
            .all(|light| light.node_id != hidden_point)
    );
    assert!(
        packet
            .scene
            .rect_lights
            .iter()
            .any(|light| light.node_id == visible_rect)
    );
    assert!(
        packet
            .scene
            .rect_lights
            .iter()
            .all(|light| light.node_id != hidden_rect)
    );
    assert!(
        packet
            .scene
            .spot_lights
            .iter()
            .any(|light| light.node_id == visible_spot)
    );
    assert!(
        packet
            .scene
            .spot_lights
            .iter()
            .all(|light| light.node_id != hidden_spot)
    );
}

#[test]
fn explicit_camera_request_layers_override_scene_camera_layers_for_direct_frame_extract() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world.set_active_camera(camera);
    let request_visible_mesh = spawn_mesh_on_layer(&mut world, 0b0100, Mobility::Dynamic);
    let scene_camera_visible_mesh = spawn_mesh_on_layer(&mut world, 0b0010, Mobility::Dynamic);
    let request_visible_sprite = spawn_sprite_on_layer(&mut world, 0b0100);
    let scene_camera_visible_sprite = spawn_sprite_on_layer(&mut world, 0b0010);
    let request_visible_light =
        spawn_light_on_layer(&mut world, NodeKind::DirectionalLight, 0b0100);
    let scene_camera_visible_light =
        spawn_light_on_layer(&mut world, NodeKind::DirectionalLight, 0b0010);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(705),
        SceneViewportExtractRequest {
            camera: Some(camera_descriptor_with_layers(0b0100)),
            ..SceneViewportExtractRequest::default()
        },
    ));

    assert!(
        extract
            .geometry
            .meshes
            .iter()
            .any(|mesh| mesh.node_id == request_visible_mesh)
    );
    assert!(
        extract
            .geometry
            .meshes
            .iter()
            .all(|mesh| mesh.node_id != scene_camera_visible_mesh)
    );
    assert!(
        extract
            .sprites
            .sprites
            .iter()
            .any(|sprite| sprite.entity == request_visible_sprite)
    );
    assert!(
        extract
            .sprites
            .sprites
            .iter()
            .all(|sprite| sprite.entity != scene_camera_visible_sprite)
    );
    assert!(
        extract
            .view
            .selected_camera_layers()
            .intersects_scene_schema_v1_mask(0b0100)
    );
    assert!(
        extract
            .visibility
            .renderable_entities
            .contains(&request_visible_mesh)
    );
    assert!(
        extract
            .visibility
            .renderable_entities
            .contains(&request_visible_sprite)
    );
    assert!(
        !extract
            .visibility
            .renderable_entities
            .contains(&scene_camera_visible_mesh)
    );
    assert!(
        !extract
            .visibility
            .renderable_entities
            .contains(&scene_camera_visible_sprite)
    );
    assert!(
        extract
            .lighting
            .directional_lights
            .iter()
            .any(|light| light.node_id == request_visible_light)
    );
    assert!(
        extract
            .lighting
            .directional_lights
            .iter()
            .all(|light| light.node_id != scene_camera_visible_light)
    );
}

#[test]
fn render_frame_extract_carries_scene_post_process_volumes_for_camera_layers() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world.set_active_camera(camera);
    let visible_volume = spawn_post_process_volume_on_layer(
        &mut world,
        0b0010,
        PostProcessVolumeComponent::global(
            8.0,
            RenderPostProcessVolumeProfile::default()
                .with_bloom(RenderBloomSettings {
                    intensity: 0.75,
                    radius: 0.4,
                    ..RenderBloomSettings::default()
                })
                .with_color_grading(RenderColorGradingSettings {
                    exposure: 1.4,
                    ..RenderColorGradingSettings::default()
                })
                .with_effect_stack(
                    crate::core::framework::render::RenderPostProcessEffectStackSettings {
                        tonemap: RenderTonemapSettings {
                            operator: RenderTonemapOperator::Aces,
                            ..RenderTonemapSettings::default()
                        },
                        ..Default::default()
                    },
                ),
        ),
    );
    let _hidden_volume = spawn_post_process_volume_on_layer(
        &mut world,
        0b0100,
        PostProcessVolumeComponent::global(
            16.0,
            RenderPostProcessVolumeProfile::default().with_bloom(RenderBloomSettings {
                intensity: 5.0,
                ..RenderBloomSettings::default()
            }),
        ),
    );

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(708),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.post_process.volumes.len(), 1);
    let volume = &extract.post_process.volumes[0];
    assert_eq!(volume.priority, 8.0);
    assert!(volume.volume_mask.intersects_scene_schema_v1_mask(0b0010));
    let resolved = extract
        .post_process
        .resolved_settings_for_camera(
            extract.view.camera.transform.translation,
            extract.view.selected_camera_volume_layers(),
        )
        .expect("planned volume evaluation should resolve");
    assert_eq!(resolved.bloom.intensity, 0.75);
    assert_eq!(resolved.color_grading.exposure, 1.4);
    assert_eq!(
        resolved.effect_stack.tonemap.operator,
        RenderTonemapOperator::Aces
    );
    assert!(
        extract
            .post_process
            .volumes
            .iter()
            .all(|volume| volume.priority != 16.0)
    );
    assert!(
        world
            .get::<PostProcessVolumeComponent>(visible_volume)
            .is_some()
    );
}

#[test]
fn inactive_post_process_volume_hierarchy_is_excluded_from_frame_extract() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world.set_active_camera(camera);
    let parent = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let volume = spawn_post_process_volume_on_layer(
        &mut world,
        0b0010,
        PostProcessVolumeComponent::global(
            4.0,
            RenderPostProcessVolumeProfile::default().with_bloom(RenderBloomSettings {
                intensity: 0.9,
                ..RenderBloomSettings::default()
            }),
        ),
    );
    world.set_parent_checked(volume, Some(parent)).unwrap();
    world.set_active_self(parent, false).unwrap();

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(709),
        SceneViewportExtractRequest::default(),
    ));

    assert!(extract.post_process.volumes.is_empty());
    let resolved = extract
        .post_process
        .resolved_settings_for_camera(
            extract.view.camera.transform.translation,
            extract.view.selected_camera_volume_layers(),
        )
        .expect("planned volume evaluation should resolve");
    assert_eq!(resolved.bloom, RenderBloomSettings::default());
}
