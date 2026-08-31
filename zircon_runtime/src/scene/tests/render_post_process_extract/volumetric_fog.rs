use super::*;

#[test]
fn render_volumetric_explicit_camera_uses_culling_mask_for_local_fog_volumes() {
    let mut world = World::empty();
    let volume = spawn_local_volumetric_box(&mut world, 0b0010, Vec3::ZERO, Vec3::ONE, 1.0);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(813),
        SceneViewportExtractRequest {
            camera: Some(camera_descriptor_with_culling_and_volume_layers(
                0b0010, 0b0100,
            )),
            ..SceneViewportExtractRequest::default()
        },
    ));

    assert!(extract.post_process.volumes.is_empty());
    assert_eq!(extract.lighting.advanced_lighting.fog_volumes.len(), 1);
    let fog = &extract.lighting.advanced_lighting.fog_volumes[0];
    assert_eq!(fog.volume_id, volume);
    assert_eq!(
        fog.layer_mask,
        RenderLayerSet::from_scene_schema_v1_mask(0b0010)
    );
}

#[test]
fn render_volumetric_scene_local_profile_and_light_marker_feed_advanced_extract() {
    let mut world = World::empty();
    spawn_camera_on_layer(&mut world, 0b0010);
    let volume = spawn_local_volumetric_box(
        &mut world,
        0b0010,
        Vec3::new(2.0, 3.0, 4.0),
        Vec3::new(1.0, 2.0, 3.0),
        0.5,
    );

    let light_entity = world
        .spawn_node(NodeKind::DirectionalLight)
        .expect("test scene spawn should succeed");
    world
        .insert(
            light_entity,
            DirectionalLight {
                volumetric: true,
                ..DirectionalLight::default()
            },
        )
        .unwrap();
    world.set_render_layer_mask(light_entity, 0b0010).unwrap();

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(901),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(
        extract.lighting.advanced_lighting.volumetric_light_ids,
        vec![light_entity]
    );
    assert_eq!(extract.lighting.advanced_lighting.fog_volumes.len(), 1);
    let fog = &extract.lighting.advanced_lighting.fog_volumes[0];
    assert_eq!(fog.volume_id, volume);
    assert_eq!(fog.bounds_min, Vec3::new(1.0, 1.0, 1.0));
    assert_eq!(fog.bounds_max, Vec3::new(3.0, 5.0, 7.0));
    assert_near(fog.density, 0.1);
    assert_eq!(fog.albedo, Vec3::new(0.4, 0.6, 0.8));
    assert!(
        extract.post_process.volumes[0]
            .overrides
            .iter()
            .all(|entry| entry.component_id != VOLUMETRIC_FOG_COMPONENT_ID)
    );
    assert_eq!(
        resolved_post_process_settings(&extract).volumetric_fog,
        VolumetricFogSettings::default()
    );
}

#[test]
fn render_volumetric_local_volume_rejects_invalid_bounds() {
    let mut world = World::empty();
    spawn_camera_on_layer(&mut world, 0b0010);
    spawn_local_volumetric_box(
        &mut world,
        0b0010,
        Vec3::ZERO,
        Vec3::new(0.0, 1.0, 1.0),
        1.0,
    );
    spawn_local_volumetric_box(
        &mut world,
        0b0010,
        Vec3::ZERO,
        Vec3::new(f32::NAN, 1.0, 1.0),
        1.0,
    );

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(902),
        SceneViewportExtractRequest::default(),
    ));

    assert!(extract.lighting.advanced_lighting.fog_volumes.is_empty());
}

fn spawn_local_volumetric_box(
    world: &mut World,
    layer_mask: u32,
    translation: Vec3,
    half_extents: Vec3,
    weight: f32,
) -> crate::scene::EntityId {
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .update_transform(entity, Transform::from_translation(translation))
        .unwrap();
    world
        .insert(
            entity,
            PostProcessVolumeComponent::local(
                1.0,
                weight,
                0.0,
                RenderPostProcessVolumeProfile::default().with_volumetric_fog(
                    VolumetricFogSettings {
                        density: 0.2,
                        albedo: Vec3::new(0.4, 0.6, 0.8),
                        ..VolumetricFogSettings::default()
                    },
                ),
            ),
        )
        .unwrap();
    world
        .insert(
            entity,
            ColliderComponent {
                shape: ColliderShape::Box { half_extents },
                sensor: true,
                ..ColliderComponent::default()
            },
        )
        .unwrap();
    world.set_render_layer_mask(entity, layer_mask).unwrap();
    entity
}
