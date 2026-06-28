use crate::asset::{
    SceneAsset, SceneBloomSettingsAsset, SceneCameraAsset, SceneColorGradingSettingsAsset,
    SceneEntityAsset, SceneFogSettingsAsset, SceneMobilityAsset, ScenePostProcessEffectStackAsset,
    ScenePostProcessSettingsAsset, ScenePostProcessVolumeAsset, ScenePostProcessVolumeProfileAsset,
    SceneTonemapOperatorAsset, SceneTonemapSettingsAsset, SceneVignetteSettingsAsset,
    TransformAsset,
};
use crate::core::framework::render::{
    CameraRenderDescriptor, RenderBloomSettings, RenderColorGradingSettings, RenderExtractContext,
    RenderFrameExtract, RenderLayerSet, RenderPostProcessEffectStackSettings,
    RenderPostProcessVolumeProfile, RenderResolvedPostProcessSettings, RenderTonemapOperator,
    RenderTonemapSettings, RenderWorldSnapshotHandle, SceneViewportExtractRequest,
    ViewportCameraSnapshot, VolumeShapeExtract,
};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::{
    CameraComponent, ColliderComponent, ColliderShape, MeshRenderer, NodeKind,
    PostProcessSettingsComponent, PostProcessVolumeComponent,
};
use crate::scene::World;

#[test]
fn scene_asset_post_process_settings_feed_render_extract() {
    let project_root = super::support::unique_temp_project_root(
        "scene_asset_post_process_settings_feed_render_extract",
    );
    let project = super::support::create_test_project(&project_root);
    let scene = SceneAsset {
        entities: vec![
            SceneEntityAsset {
                entity: 1,
                name: "MoodCamera".to_string(),
                parent: None,
                transform: TransformAsset::default(),
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: Some(SceneCameraAsset {
                    post_process_settings: Some(ScenePostProcessSettingsAsset {
                        bloom: SceneBloomSettingsAsset {
                            threshold: 0.25,
                            intensity: 0.4,
                            radius: 0.5,
                        },
                        color_grading: SceneColorGradingSettingsAsset {
                            exposure: 0.85,
                            contrast: 1.15,
                            saturation: 0.75,
                            gamma: 1.05,
                            tint: [0.72, 0.8, 1.0],
                        },
                        effect_stack: ScenePostProcessEffectStackAsset {
                            tonemap: SceneTonemapSettingsAsset {
                                operator: SceneTonemapOperatorAsset::Aces,
                                exposure_bias: -0.15,
                                white_point: 1.25,
                            },
                            vignette: SceneVignetteSettingsAsset {
                                intensity: 0.35,
                                smoothness: 0.6,
                                roundness: 0.9,
                            },
                            fog: SceneFogSettingsAsset {
                                density: 0.07,
                                height_falloff: 0.2,
                                color: [0.18, 0.22, 0.3],
                            },
                            ..ScenePostProcessEffectStackAsset::default()
                        },
                    }),
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
            },
            SceneEntityAsset {
                entity: 2,
                name: "MoodVolume".to_string(),
                parent: None,
                transform: TransformAsset::default(),
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Static,
                camera: None,
                mesh: None,
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: Some(ScenePostProcessVolumeAsset {
                    active: true,
                    is_global: true,
                    priority: 2.0,
                    weight: 0.5,
                    blend_distance: 0.0,
                    profile: ScenePostProcessVolumeProfileAsset {
                        bloom: Some(SceneBloomSettingsAsset {
                            threshold: 0.2,
                            intensity: 1.0,
                            radius: 0.9,
                        }),
                        color_grading: None,
                        effect_stack: Some(ScenePostProcessEffectStackAsset {
                            tonemap: SceneTonemapSettingsAsset {
                                operator: SceneTonemapOperatorAsset::Filmic,
                                exposure_bias: -0.1,
                                white_point: 1.1,
                            },
                            ..ScenePostProcessEffectStackAsset::default()
                        }),
                    },
                }),
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
            },
        ],
    };
    let mut world = World::from_scene_asset(&project, &scene).unwrap();

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(800),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.view.scene_camera_entity, Some(1));
    assert_near(extract.post_process.bloom.intensity, 0.4);
    assert_near(extract.post_process.color_grading.saturation, 0.75);
    assert_eq!(
        extract.post_process.effect_stack.tonemap.operator,
        RenderTonemapOperator::Aces
    );
    assert_near(extract.post_process.effect_stack.vignette.intensity, 0.35);
    assert_near(extract.post_process.effect_stack.fog.density, 0.07);
    assert_eq!(extract.post_process.volumes.len(), 1);

    let resolved = resolved_post_process_settings(&extract);
    assert_near(resolved.bloom.intensity, 0.7);
    assert_eq!(
        resolved.effect_stack.tonemap.operator,
        RenderTonemapOperator::Filmic
    );
}

#[test]
fn scene_camera_post_process_settings_seed_frame_extract_before_volume_resolution() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    let settings = camera_post_process_settings();
    world.insert(camera, settings.clone()).unwrap();
    world
        .insert(
            camera,
            PostProcessVolumeComponent::global(
                1.0,
                RenderPostProcessVolumeProfile::default()
                    .with_bloom(RenderBloomSettings {
                        intensity: 1.0,
                        radius: 0.8,
                        ..RenderBloomSettings::default()
                    })
                    .with_effect_stack(RenderPostProcessEffectStackSettings {
                        tonemap: RenderTonemapSettings {
                            operator: RenderTonemapOperator::Aces,
                            white_point: 1.4,
                            ..RenderTonemapSettings::default()
                        },
                        ..RenderPostProcessEffectStackSettings::default()
                    }),
            )
            .with_weight(0.5),
        )
        .unwrap();

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(801),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.view.scene_camera_entity, Some(camera));
    assert_eq!(extract.post_process.bloom, settings.bloom);
    assert_eq!(extract.post_process.color_grading, settings.color_grading);
    assert_eq!(extract.post_process.effect_stack, settings.effect_stack);
    assert_eq!(extract.post_process.volumes.len(), 1);

    let resolved = resolved_post_process_settings(&extract);
    assert_near(resolved.bloom.intensity, 0.7);
    assert_near(resolved.bloom.radius, 0.65);
    assert_eq!(
        resolved.effect_stack.tonemap.operator,
        RenderTonemapOperator::Aces
    );
    assert_near(resolved.effect_stack.tonemap.white_point, 1.25);
}

#[test]
fn explicit_request_camera_ignores_scene_camera_post_process_settings() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world
        .insert(camera, camera_post_process_settings())
        .expect("scene camera should accept post-process settings");

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(802),
        SceneViewportExtractRequest {
            camera: Some(camera_descriptor_with_layers(0b0010)),
            ..SceneViewportExtractRequest::default()
        },
    ));

    assert_eq!(extract.view.scene_camera_entity, None);
    assert_eq!(extract.post_process.bloom, RenderBloomSettings::default());
    assert_eq!(
        extract.post_process.color_grading,
        RenderColorGradingSettings::default()
    );
    assert_eq!(
        extract.post_process.effect_stack,
        RenderPostProcessEffectStackSettings::default()
    );
}

#[test]
fn explicit_request_camera_uses_volume_mask_for_post_process_volumes() {
    let mut world = World::empty();
    spawn_global_volume_on_layer(&mut world, 0b0100);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(812),
        SceneViewportExtractRequest {
            camera: Some(camera_descriptor_with_culling_and_volume_layers(
                0b0010, 0b0100,
            )),
            ..SceneViewportExtractRequest::default()
        },
    ));

    assert_eq!(
        *extract.view.selected_camera_layers(),
        RenderLayerSet::from_scene_schema_v1_mask(0b0010)
    );
    assert_eq!(
        *extract.view.selected_camera_volume_layers(),
        RenderLayerSet::from_scene_schema_v1_mask(0b0100)
    );
    assert_eq!(extract.post_process.volumes.len(), 1);
    assert_eq!(
        extract.post_process.volumes[0].volume_mask,
        RenderLayerSet::from_scene_schema_v1_mask(0b0100)
    );
    assert_near(
        resolved_post_process_settings(&extract).bloom.intensity,
        1.0,
    );
}

#[test]
fn local_sphere_post_process_volume_uses_camera_distance_for_full_influence() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world
        .update_transform(
            camera,
            Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
        )
        .unwrap();
    let volume = spawn_local_sphere_volume(&mut world, 0b0010, Vec3::ZERO, 1.0, 2.0);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(803),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.post_process.volumes.len(), 1);
    assert_eq!(
        extract.post_process.volumes[0].shape,
        VolumeShapeExtract::Sphere {
            center: Vec3::ZERO,
            radius: 1.0,
            blend_distance: 2.0,
        }
    );
    assert!(extract.post_process.volumes[0]
        .overrides
        .iter()
        .any(|override_entry| override_entry.component_id == "post.bloom"));
    assert_eq!(extract.post_process.volumes[0].priority, 2.0);
    assert_near(
        resolved_post_process_settings(&extract).bloom.intensity,
        1.0,
    );
    assert!(world.get::<PostProcessVolumeComponent>(volume).is_some());
}

#[test]
fn local_sphere_post_process_volume_fades_in_blend_band() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world
        .update_transform(
            camera,
            Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
        )
        .unwrap();
    spawn_local_sphere_volume(&mut world, 0b0010, Vec3::ZERO, 1.0, 1.0);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(804),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.post_process.volumes.len(), 1);
    assert_near(
        resolved_post_process_settings(&extract).bloom.intensity,
        0.75,
    );
}

#[test]
fn local_sphere_post_process_volume_outside_blend_band_has_zero_influence() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world
        .update_transform(
            camera,
            Transform::from_translation(Vec3::new(3.5, 0.0, 0.0)),
        )
        .unwrap();
    spawn_local_sphere_volume(&mut world, 0b0010, Vec3::ZERO, 1.0, 1.0);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(805),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.post_process.volumes.len(), 1);
    assert_eq!(
        extract.post_process.volumes[0].shape,
        VolumeShapeExtract::Sphere {
            center: Vec3::ZERO,
            radius: 1.0,
            blend_distance: 2.0,
        }
    );
    assert_eq!(
        resolved_post_process_settings(&extract).bloom,
        RenderBloomSettings::default()
    );
}

#[test]
fn local_box_post_process_volume_uses_camera_distance_for_blend() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world
        .update_transform(
            camera,
            Transform::from_translation(Vec3::new(2.0, 0.5, 0.25)),
        )
        .unwrap();
    spawn_local_volume(
        &mut world,
        0b0010,
        Vec3::ZERO,
        1.0,
        2.0,
        Some(ColliderShape::Box {
            half_extents: Vec3::splat(1.0),
        }),
    );

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(806),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.post_process.volumes.len(), 1);
    assert_eq!(
        extract.post_process.volumes[0].shape,
        VolumeShapeExtract::Box {
            center: Vec3::ZERO,
            half_extents: Vec3::splat(1.0),
            rotation: crate::core::math::Quat::IDENTITY,
            blend_distance: 2.0,
        }
    );
    assert_near(
        resolved_post_process_settings(&extract).bloom.intensity,
        0.75,
    );
}

#[test]
fn local_capsule_post_process_volume_is_not_projected_to_planned_extract() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world
        .update_transform(
            camera,
            Transform::from_translation(Vec3::new(0.0, 2.5, 0.0)),
        )
        .unwrap();
    spawn_local_volume(
        &mut world,
        0b0010,
        Vec3::ZERO,
        1.0,
        1.0,
        Some(ColliderShape::Capsule {
            radius: 1.0,
            half_height: 1.0,
        }),
    );

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(807),
        SceneViewportExtractRequest::default(),
    ));

    assert!(extract.post_process.volumes.is_empty());
    assert_eq!(
        resolved_post_process_settings(&extract).bloom,
        RenderBloomSettings::default()
    );
}

#[test]
fn local_post_process_volume_without_collider_is_excluded() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world
        .update_transform(camera, Transform::from_translation(Vec3::ZERO))
        .unwrap();
    let volume = spawn_local_volume(&mut world, 0b0010, Vec3::ZERO, 1.0, 2.0, None);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(808),
        SceneViewportExtractRequest::default(),
    ));

    assert!(world.get::<PostProcessVolumeComponent>(volume).is_some());
    assert!(extract.post_process.volumes.is_empty());
    assert_eq!(
        resolved_post_process_settings(&extract).bloom,
        RenderBloomSettings::default()
    );
}

fn resolved_post_process_settings(
    extract: &RenderFrameExtract,
) -> RenderResolvedPostProcessSettings {
    extract
        .post_process
        .resolved_settings_for_camera(
            extract.view.camera.transform.translation,
            extract.view.selected_camera_volume_layers(),
        )
        .expect("planned volume evaluation should resolve")
}

fn camera_descriptor_with_layers(mask: u32) -> CameraRenderDescriptor {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    camera.culling_mask = RenderLayerSet::from_scene_schema_v1_mask(mask);
    camera.volume_mask = camera.culling_mask.clone();
    camera
}

fn camera_descriptor_with_culling_and_volume_layers(
    culling_mask: u32,
    volume_mask: u32,
) -> CameraRenderDescriptor {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    camera.culling_mask = RenderLayerSet::from_scene_schema_v1_mask(culling_mask);
    camera.volume_mask = RenderLayerSet::from_scene_schema_v1_mask(volume_mask);
    camera
}

fn camera_post_process_settings() -> PostProcessSettingsComponent {
    PostProcessSettingsComponent::from_parts(
        RenderBloomSettings {
            threshold: 0.35,
            intensity: 0.4,
            radius: 0.5,
        },
        RenderColorGradingSettings {
            exposure: 1.2,
            contrast: 1.1,
            saturation: 0.9,
            gamma: 1.05,
            tint: Vec3::new(1.0, 0.95, 0.9),
        },
        RenderPostProcessEffectStackSettings {
            tonemap: RenderTonemapSettings {
                operator: RenderTonemapOperator::Filmic,
                exposure_bias: 0.2,
                white_point: 1.1,
            },
            ..RenderPostProcessEffectStackSettings::default()
        },
    )
}

fn spawn_local_sphere_volume(
    world: &mut World,
    layer_mask: u32,
    translation: Vec3,
    radius: f32,
    priority: f32,
) -> crate::scene::EntityId {
    spawn_local_volume(
        world,
        layer_mask,
        translation,
        priority,
        2.0,
        Some(ColliderShape::Sphere { radius }),
    )
}

fn spawn_local_volume(
    world: &mut World,
    layer_mask: u32,
    translation: Vec3,
    priority: f32,
    blend_distance: f32,
    collider_shape: Option<ColliderShape>,
) -> crate::scene::EntityId {
    let entity = world.spawn_node(NodeKind::Mesh);
    let _ = world.remove::<MeshRenderer>(entity).unwrap();
    world
        .update_transform(entity, Transform::from_translation(translation))
        .unwrap();
    world
        .insert(
            entity,
            PostProcessVolumeComponent::local(
                priority,
                1.0,
                blend_distance,
                RenderPostProcessVolumeProfile::default().with_bloom(RenderBloomSettings {
                    intensity: 1.0,
                    ..RenderBloomSettings::default()
                }),
            ),
        )
        .unwrap();
    if let Some(shape) = collider_shape {
        world
            .insert(
                entity,
                ColliderComponent {
                    shape,
                    sensor: true,
                    ..ColliderComponent::default()
                },
            )
            .unwrap();
    }
    world.set_render_layer_mask(entity, layer_mask).unwrap();
    entity
}

fn spawn_global_volume_on_layer(world: &mut World, layer_mask: u32) -> crate::scene::EntityId {
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .insert(
            entity,
            PostProcessVolumeComponent::global(
                0.0,
                RenderPostProcessVolumeProfile::default().with_bloom(RenderBloomSettings {
                    intensity: 1.0,
                    ..RenderBloomSettings::default()
                }),
            ),
        )
        .unwrap();
    world.set_render_layer_mask(entity, layer_mask).unwrap();
    entity
}

fn spawn_camera_on_layer(world: &mut World, layer_mask: u32) -> crate::scene::EntityId {
    let camera = world.spawn_node(NodeKind::Camera);
    world.set_active_camera(camera);
    world.set_render_layer_mask(camera, layer_mask).unwrap();
    assert!(world.get::<CameraComponent>(camera).is_some());
    camera
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.0001,
        "expected {actual} to be near {expected}"
    );
}
