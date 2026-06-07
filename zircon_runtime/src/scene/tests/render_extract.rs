use std::collections::BTreeMap;

use crate::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use crate::core::framework::render::{
    DisplayMode, RenderBloomSettings, RenderCameraOrderAmbiguity, RenderCameraTarget,
    RenderCameraTargetOrderKey, RenderColorGradingSettings, RenderExtractContext,
    RenderExtractProducer, RenderLayerSet, RenderMaterialAlphaMode, RenderPhase,
    RenderPhaseMeshSource, RenderPostProcessVolumeProfile, RenderTonemapOperator,
    RenderTonemapSettings, RenderVirtualGeometryDebugState, RenderWorldSnapshotHandle,
    SceneViewportExtractRequest, ViewportCameraSnapshot, ViewportRenderSettings,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{AnimationSkeletonMarker, ResourceHandle, ResourceId, TextureMarker};
use crate::scene::components::{
    AmbientLight, AnimationSkeletonComponent, CameraComponent, MeshRenderer, Mobility, NodeKind,
    PostProcessVolumeComponent, Sprite2dComponent,
};
use crate::scene::{DefaultLevelManager, World};

use super::support::{material_handle, model_handle};

#[test]
fn world_render_frame_extract_populates_direct_renderer_sections() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0111);
    world.set_active_camera(camera);
    let dynamic_mesh = spawn_mesh_on_layer(&mut world, 0b0010, Mobility::Dynamic);
    let static_mesh = spawn_mesh_on_layer(&mut world, 0b0100, Mobility::Static);
    let sprite = spawn_sprite_on_layer(&mut world, 0b0010);
    let ambient = world.spawn_node(NodeKind::AmbientLight);
    let directional = world.spawn_node(NodeKind::DirectionalLight);
    let point = world.spawn_node(NodeKind::PointLight);
    let rect = world.spawn_node(NodeKind::RectLight);
    let spot = world.spawn_node(NodeKind::SpotLight);

    world
        .update_transform(
            dynamic_mesh,
            Transform::from_translation(Vec3::new(2.0, 3.0, 4.0)),
        )
        .unwrap();
    world
        .get_mut::<MeshRenderer>(dynamic_mesh)
        .unwrap()
        .morph_weights = vec![0.25, 0.75];
    world.get_mut::<MeshRenderer>(dynamic_mesh).unwrap().tint = Vec4::new(0.2, 0.4, 0.6, 1.0);
    world
        .get_mut::<MeshRenderer>(static_mesh)
        .unwrap()
        .material_alpha_mode = RenderMaterialAlphaMode::Blend;
    world
        .update_transform(point, Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)))
        .unwrap();

    assert!(world.has_pending_scene_systems());
    let debug = RenderVirtualGeometryDebugState {
        forced_mip: Some(4),
        visualize_bvh: true,
        visualize_visbuffer: true,
        ..RenderVirtualGeometryDebugState::default()
    };
    let context = RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(701),
        SceneViewportExtractRequest {
            settings: ViewportRenderSettings {
                display_mode: DisplayMode::WireOverlay,
                preview_lighting: false,
                preview_skybox: false,
                ..ViewportRenderSettings::default()
            },
            viewport_size: Some(UVec2::new(1920, 1080)),
            virtual_geometry_debug: Some(debug),
            ..SceneViewportExtractRequest::default()
        },
    );

    let extract = world.build_prepared_render_frame_extract(&context);

    assert_eq!(extract.world.raw(), 701);
    assert_eq!(extract.view.camera.aspect_ratio, 1920.0 / 1080.0);
    assert_eq!(
        extract.view.camera.render_layers,
        RenderLayerSet::from_legacy_mask(0b0111)
    );

    let dynamic_index = extract
        .geometry
        .meshes
        .iter()
        .position(|mesh| mesh.node_id == dynamic_mesh)
        .expect("dynamic mesh should be extracted directly");
    let dynamic_row = &extract.geometry.meshes[dynamic_index];
    assert_eq!(dynamic_row.transform.translation, Vec3::new(2.0, 3.0, 4.0));
    assert_eq!(dynamic_row.model, model_handle("res://models/direct-2.obj"));
    assert_eq!(
        dynamic_row.material,
        material_handle("res://materials/direct-2.zmaterial")
    );
    assert_eq!(dynamic_row.morph_weights, vec![0.25, 0.75]);
    assert_eq!(dynamic_row.tint, Vec4::new(0.2, 0.4, 0.6, 1.0));
    assert_eq!(dynamic_row.render_layer_mask, 0b0010);

    let static_index = extract
        .geometry
        .meshes
        .iter()
        .position(|mesh| mesh.node_id == static_mesh)
        .expect("static mesh should be extracted directly");
    assert!(extract.geometry.phase_inputs.iter().any(|input| {
        input.entity == dynamic_mesh
            && input.mesh_index == dynamic_index
            && input.material_alpha_mode == RenderMaterialAlphaMode::Opaque
    }));
    assert!(extract.geometry.phase_inputs.iter().any(|input| {
        input.entity == static_mesh
            && input.mesh_index == static_index
            && input.material_alpha_mode == RenderMaterialAlphaMode::Blend
    }));
    assert!(extract
        .geometry
        .phase_queue
        .items_for_phase(RenderPhase::Transparent3d)
        .any(|item| item.mesh_source == RenderPhaseMeshSource::MeshIndex(static_index)));

    assert_eq!(extract.sprites.sprites.len(), 1);
    assert_eq!(extract.sprites.sprites[0].entity, sprite);
    assert!(extract
        .sprites
        .phase_queue
        .items_for_phase(RenderPhase::Transparent3d)
        .any(|item| item.mesh_source == RenderPhaseMeshSource::SpriteIndex(0)));

    assert_eq!(extract.lighting.ambient_lights.len(), 1);
    assert_eq!(extract.lighting.directional_lights.len(), 1);
    assert!(extract
        .lighting
        .point_lights
        .iter()
        .any(|light| light.node_id == point && light.position == Vec3::new(1.0, 2.0, 3.0)));
    assert!(extract
        .lighting
        .rect_lights
        .iter()
        .any(|light| light.node_id == rect));
    assert!(extract
        .lighting
        .spot_lights
        .iter()
        .any(|light| light.node_id == spot));
    assert!(extract
        .lighting
        .directional_lights
        .iter()
        .any(|light| light.node_id == directional));
    assert!(extract
        .lighting
        .ambient_lights
        .iter()
        .any(|light| light.color == AmbientLight::default().color));
    assert!(extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .is_some_and(|gi| !gi.enabled));

    assert_eq!(extract.post_process.display_mode, DisplayMode::WireOverlay);
    assert!(!extract.post_process.preview.lighting_enabled);
    assert!(!extract.post_process.preview.skybox_enabled);
    assert_eq!(extract.post_process.bloom.intensity, 0.0);
    assert_eq!(extract.post_process.color_grading.exposure, 1.0);
    assert!(!extract.post_process.stack.initial_resources.is_empty());
    assert!(extract
        .post_process
        .graph
        .final_composite_node
        .as_deref()
        .is_some_and(|node| node == "final-composite"));
    assert_eq!(extract.geometry.virtual_geometry_debug, Some(debug));
    let virtual_geometry = extract
        .geometry
        .virtual_geometry
        .as_ref()
        .expect("direct frame extract should preserve empty VG sideband shape");
    assert_eq!(virtual_geometry.debug, debug);
    assert!(virtual_geometry.clusters.is_empty());
    assert_eq!(virtual_geometry.cluster_budget, 0);

    assert!(extract
        .visibility
        .renderable_entities
        .contains(&dynamic_mesh));
    assert!(extract
        .visibility
        .renderable_entities
        .contains(&static_mesh));
    assert!(extract.visibility.renderable_entities.contains(&sprite));
    assert!(extract.visibility.dynamic_entities.contains(&dynamic_mesh));
    assert!(extract.visibility.dynamic_entities.contains(&sprite));
    assert!(extract.visibility.static_entities.contains(&static_mesh));
    assert_eq!(
        extract.visibility.renderables.len(),
        extract.geometry.meshes.len() + 1
    );
    assert!(!world.has_pending_scene_systems());

    let _ = ambient;
}

#[test]
fn inactive_camera_render_frame_extract_keeps_view_but_removes_scene_payload() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b1111);
    world.set_active_camera(camera);
    world.get_mut::<CameraComponent>(camera).unwrap().is_active = false;
    spawn_mesh_on_layer(&mut world, 0b0001, Mobility::Dynamic);
    spawn_sprite_on_layer(&mut world, 0b0001);
    world.spawn_node(NodeKind::AmbientLight);
    world.spawn_node(NodeKind::DirectionalLight);
    world.spawn_node(NodeKind::PointLight);
    world.spawn_node(NodeKind::RectLight);
    world.spawn_node(NodeKind::SpotLight);

    let debug = RenderVirtualGeometryDebugState {
        freeze_cull: true,
        ..RenderVirtualGeometryDebugState::default()
    };
    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(702),
        SceneViewportExtractRequest {
            settings: ViewportRenderSettings {
                display_mode: DisplayMode::WireOnly,
                ..ViewportRenderSettings::default()
            },
            virtual_geometry_debug: Some(debug),
            ..SceneViewportExtractRequest::default()
        },
    ));

    assert!(!extract.view.camera.is_active);
    assert!(extract.geometry.meshes.is_empty());
    assert!(extract.geometry.phase_inputs.is_empty());
    assert!(extract.geometry.phase_queue.items.is_empty());
    assert!(extract.sprites.sprites.is_empty());
    assert!(extract.sprites.phase_queue.items.is_empty());
    assert!(extract.lighting.ambient_lights.is_empty());
    assert!(extract.lighting.directional_lights.is_empty());
    assert!(extract.lighting.point_lights.is_empty());
    assert!(extract.lighting.rect_lights.is_empty());
    assert!(extract.lighting.spot_lights.is_empty());
    assert!(extract.visibility.renderable_entities.is_empty());
    assert!(extract.visibility.renderables.is_empty());
    assert_eq!(extract.post_process.display_mode, DisplayMode::WireOnly);
    assert_eq!(extract.geometry.virtual_geometry_debug, Some(debug));
    assert!(extract
        .geometry
        .virtual_geometry
        .as_ref()
        .is_some_and(|vg| vg.debug == debug));
    assert!(extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .is_some_and(|gi| !gi.enabled));
    assert!(extract.particles.emitters.is_empty());
}

#[test]
fn hierarchy_inactive_camera_render_frame_extract_keeps_view_but_removes_scene_payload() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Cube);
    let camera = spawn_camera_on_layer(&mut world, 0b1111);
    world.set_parent_checked(camera, Some(parent)).unwrap();
    world.set_active_self(parent, false).unwrap();
    world.set_active_camera(camera);
    spawn_mesh_on_layer(&mut world, 0b0001, Mobility::Dynamic);
    spawn_sprite_on_layer(&mut world, 0b0001);
    world.spawn_node(NodeKind::DirectionalLight);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(703),
        SceneViewportExtractRequest::default(),
    ));

    assert!(!extract.view.camera.is_active);
    assert!(extract.geometry.meshes.is_empty());
    assert!(extract.geometry.phase_inputs.is_empty());
    assert!(extract.sprites.sprites.is_empty());
    assert!(extract.lighting.directional_lights.is_empty());
    assert!(extract.visibility.renderable_entities.is_empty());
}

#[test]
fn render_frame_extract_filters_meshes_sprites_and_visibility_by_camera_layers() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world.set_active_camera(camera);
    let visible_mesh = spawn_mesh_on_layer(&mut world, 0b0010, Mobility::Static);
    let hidden_mesh = spawn_mesh_on_layer(&mut world, 0b0100, Mobility::Dynamic);
    let visible_sprite = spawn_sprite_on_layer(&mut world, 0b0010);
    let hidden_sprite = spawn_sprite_on_layer(&mut world, 0b0100);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(703),
        SceneViewportExtractRequest::default(),
    ));

    assert!(extract
        .geometry
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == visible_mesh));
    assert!(extract
        .geometry
        .meshes
        .iter()
        .all(|mesh| mesh.node_id != hidden_mesh));
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
        .visibility
        .renderables
        .iter()
        .all(|renderable| renderable.render_layer_mask & 0b0010 != 0));
    assert!(extract.visibility.static_entities.contains(&visible_mesh));
    assert!(extract
        .visibility
        .dynamic_entities
        .contains(&visible_sprite));
    assert!(!extract
        .visibility
        .renderable_entities
        .contains(&hidden_mesh));
    assert!(!extract
        .visibility
        .renderable_entities
        .contains(&hidden_sprite));
}

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
    assert!(extract
        .lighting
        .directional_lights
        .iter()
        .any(|light| light.node_id == visible_directional));
    assert!(extract
        .lighting
        .directional_lights
        .iter()
        .all(|light| light.node_id != hidden_directional));
    assert!(extract
        .lighting
        .point_lights
        .iter()
        .any(|light| light.node_id == visible_point));
    assert!(extract
        .lighting
        .point_lights
        .iter()
        .all(|light| light.node_id != hidden_point));
    assert!(extract
        .lighting
        .rect_lights
        .iter()
        .any(|light| light.node_id == visible_rect));
    assert!(extract
        .lighting
        .rect_lights
        .iter()
        .all(|light| light.node_id != hidden_rect));
    assert!(extract
        .lighting
        .spot_lights
        .iter()
        .any(|light| light.node_id == visible_spot));
    assert!(extract
        .lighting
        .spot_lights
        .iter()
        .all(|light| light.node_id != hidden_spot));

    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest::default());
    assert_eq!(packet.scene.ambient_lights.len(), 1);
    assert_eq!(packet.scene.ambient_lights[0].color, visible_ambient_color);
    assert!(packet
        .scene
        .directional_lights
        .iter()
        .any(|light| light.node_id == visible_directional));
    assert!(packet
        .scene
        .directional_lights
        .iter()
        .all(|light| light.node_id != hidden_directional));
    assert!(packet
        .scene
        .point_lights
        .iter()
        .any(|light| light.node_id == visible_point));
    assert!(packet
        .scene
        .point_lights
        .iter()
        .all(|light| light.node_id != hidden_point));
    assert!(packet
        .scene
        .rect_lights
        .iter()
        .any(|light| light.node_id == visible_rect));
    assert!(packet
        .scene
        .rect_lights
        .iter()
        .all(|light| light.node_id != hidden_rect));
    assert!(packet
        .scene
        .spot_lights
        .iter()
        .any(|light| light.node_id == visible_spot));
    assert!(packet
        .scene
        .spot_lights
        .iter()
        .all(|light| light.node_id != hidden_spot));
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
            camera: Some(ViewportCameraSnapshot {
                render_layers: RenderLayerSet::from_legacy_mask(0b0100),
                ..ViewportCameraSnapshot::default()
            }),
            ..SceneViewportExtractRequest::default()
        },
    ));

    assert!(extract
        .geometry
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == request_visible_mesh));
    assert!(extract
        .geometry
        .meshes
        .iter()
        .all(|mesh| mesh.node_id != scene_camera_visible_mesh));
    assert!(extract
        .sprites
        .sprites
        .iter()
        .any(|sprite| sprite.entity == request_visible_sprite));
    assert!(extract
        .sprites
        .sprites
        .iter()
        .all(|sprite| sprite.entity != scene_camera_visible_sprite));
    assert!(extract
        .view
        .camera
        .render_layers
        .intersects_legacy_mask(0b0100));
    assert!(extract
        .visibility
        .renderable_entities
        .contains(&request_visible_mesh));
    assert!(extract
        .visibility
        .renderable_entities
        .contains(&request_visible_sprite));
    assert!(!extract
        .visibility
        .renderable_entities
        .contains(&scene_camera_visible_mesh));
    assert!(!extract
        .visibility
        .renderable_entities
        .contains(&scene_camera_visible_sprite));
    assert!(extract
        .lighting
        .directional_lights
        .iter()
        .any(|light| light.node_id == request_visible_light));
    assert!(extract
        .lighting
        .directional_lights
        .iter()
        .all(|light| light.node_id != scene_camera_visible_light));
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

    assert_eq!(extract.post_process.volume_stack.volumes.len(), 1);
    let volume = &extract.post_process.volume_stack.volumes[0];
    assert_eq!(volume.priority, 8.0);
    assert!(volume.layer_mask.intersects_legacy_mask(0b0010));
    let resolved = extract
        .post_process
        .resolved_settings_for_layers(&extract.view.camera.render_layers);
    assert_eq!(resolved.bloom.intensity, 0.75);
    assert_eq!(resolved.color_grading.exposure, 1.4);
    assert_eq!(
        resolved.effect_stack.tonemap.operator,
        RenderTonemapOperator::Aces
    );
    assert!(extract
        .post_process
        .volume_stack
        .volumes
        .iter()
        .all(|volume| volume.priority != 16.0));
    assert!(world
        .get::<PostProcessVolumeComponent>(visible_volume)
        .is_some());
}

#[test]
fn inactive_post_process_volume_hierarchy_is_excluded_from_frame_extract() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world.set_active_camera(camera);
    let parent = world.spawn_node(NodeKind::Mesh);
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

    assert!(extract.post_process.volume_stack.volumes.is_empty());
    let resolved = extract
        .post_process
        .resolved_settings_for_layers(&extract.view.camera.render_layers);
    assert_eq!(resolved.bloom, RenderBloomSettings::default());
}

#[test]
fn world_render_camera_order_report_projects_active_scene_cameras() {
    let mut world = World::empty();
    let hidden_parent = world.spawn_node(NodeKind::Cube);
    world.set_active_self(hidden_parent, false).unwrap();

    let primary_a = spawn_camera_on_layer(&mut world, 0b0001);
    world.get_mut::<CameraComponent>(primary_a).unwrap().order = 1;

    let primary_b = spawn_camera_on_layer(&mut world, 0b0001);
    world.get_mut::<CameraComponent>(primary_b).unwrap().order = 1;

    let texture_camera = spawn_camera_on_layer(&mut world, 0b0010);
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "res://textures/camera-order-target.png",
    ));
    {
        let component = world.get_mut::<CameraComponent>(texture_camera).unwrap();
        component.order = -1;
        component.target = RenderCameraTarget::Texture(texture);
        component.hdr = true;
    }

    let headless_camera = spawn_camera_on_layer(&mut world, 0b0100);
    {
        let component = world.get_mut::<CameraComponent>(headless_camera).unwrap();
        component.order = 2;
        component.target = RenderCameraTarget::Headless {
            size: UVec2::new(320, 180),
        };
    }

    let hidden_camera = spawn_camera_on_layer(&mut world, 0b1000);
    world
        .get_mut::<CameraComponent>(hidden_camera)
        .unwrap()
        .order = -2;
    world
        .set_parent_checked(hidden_camera, Some(hidden_parent))
        .unwrap();

    let report = world.render_camera_order_report();

    assert_eq!(
        report
            .cameras
            .iter()
            .map(|camera| camera.entity)
            .collect::<Vec<_>>(),
        vec![texture_camera, primary_a, primary_b, headless_camera]
    );
    assert_eq!(
        report
            .cameras
            .iter()
            .map(|camera| camera.sorted_camera_index_for_target)
            .collect::<Vec<_>>(),
        vec![0, 0, 1, 0]
    );
    assert_eq!(
        report.ambiguities,
        vec![RenderCameraOrderAmbiguity {
            order: 1,
            target: RenderCameraTargetOrderKey::PrimarySurface,
        }]
    );
}

#[test]
fn render_frame_extract_carries_scene_camera_order_report_for_scene_camera() {
    let mut world = World::empty();
    let primary_a = spawn_camera_on_layer(&mut world, 0b0001);
    world.get_mut::<CameraComponent>(primary_a).unwrap().order = 1;
    world.set_active_camera(primary_a);

    let primary_b = spawn_camera_on_layer(&mut world, 0b0001);
    world.get_mut::<CameraComponent>(primary_b).unwrap().order = 1;

    let texture_camera = spawn_camera_on_layer(&mut world, 0b0010);
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "res://textures/frame-extract-camera-target.png",
    ));
    {
        let component = world.get_mut::<CameraComponent>(texture_camera).unwrap();
        component.order = -1;
        component.target = RenderCameraTarget::Texture(texture);
        component.hdr = true;
    }

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(706),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.view.scene_camera_entity, Some(primary_a));
    let report = extract
        .view
        .scene_camera_order_report
        .as_ref()
        .expect("scene-backed extract should carry camera ordering report");
    assert_eq!(
        report
            .cameras
            .iter()
            .map(|camera| camera.entity)
            .collect::<Vec<_>>(),
        vec![texture_camera, primary_a, primary_b]
    );
    assert!(report.has_ambiguities());
    assert_eq!(
        report.ambiguities,
        vec![RenderCameraOrderAmbiguity {
            order: 1,
            target: RenderCameraTargetOrderKey::PrimarySurface,
        }]
    );
}

#[test]
fn explicit_camera_render_frame_extract_has_no_scene_camera_order_report() {
    let mut world = World::empty();
    let scene_camera = spawn_camera_on_layer(&mut world, 0b0001);
    world.set_active_camera(scene_camera);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(707),
        SceneViewportExtractRequest {
            camera: Some(ViewportCameraSnapshot {
                order: 42,
                render_layers: RenderLayerSet::from_legacy_mask(0b0100),
                ..ViewportCameraSnapshot::default()
            }),
            ..SceneViewportExtractRequest::default()
        },
    ));

    assert_eq!(extract.view.camera.order, 42);
    assert_eq!(extract.view.scene_camera_entity, None);
    assert!(extract.view.scene_camera_order_report.is_none());
}

#[test]
fn level_system_render_extract_uses_world_direct_path_and_merges_animation_poses() {
    let manager = DefaultLevelManager::default();
    let level = manager.create_level(World::empty(), Default::default());
    let (mesh_with_skeleton, mesh_without_skeleton, skeleton_handle) =
        level.with_world_mut(|world| {
            let camera = spawn_camera_on_layer(world, 0b1111);
            world.set_active_camera(camera);
            let mesh_with_skeleton = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
            let mesh_without_skeleton = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
            let skeleton_handle = ResourceHandle::<AnimationSkeletonMarker>::new(
                ResourceId::from_stable_label("res://animation/hero.skeleton.zranim"),
            );
            world
                .set_animation_skeleton(
                    mesh_with_skeleton,
                    Some(AnimationSkeletonComponent {
                        skeleton: skeleton_handle,
                    }),
                )
                .unwrap();
            (mesh_with_skeleton, mesh_without_skeleton, skeleton_handle)
        });
    let missing_entity = 99_999;
    let pose = test_pose("hip");
    level.record_animation_poses(BTreeMap::from([
        (mesh_with_skeleton, pose.clone()),
        (mesh_without_skeleton, test_pose("filtered-no-skeleton")),
        (missing_entity, test_pose("filtered-missing")),
    ]));

    let extract = RenderExtractProducer::build_render_frame_extract(
        &level,
        &RenderExtractContext::new(
            RenderWorldSnapshotHandle::new(705),
            SceneViewportExtractRequest::default(),
        ),
    );

    assert_eq!(extract.world.raw(), 705);
    assert!(extract
        .geometry
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == mesh_with_skeleton));
    assert_eq!(extract.animation_poses.len(), 1);
    assert_eq!(extract.animation_poses[0].entity, mesh_with_skeleton);
    assert_eq!(extract.animation_poses[0].skeleton, skeleton_handle.id());
    assert_eq!(extract.animation_poses[0].pose, pose);
    assert!(level.with_world(|world| !world.has_pending_scene_systems()));
}

#[test]
fn render_frame_extract_snapshot_adapters_are_not_scene_production_paths() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in [
        "src/scene/render_extract/mod.rs",
        "src/scene/world/render.rs",
        "src/scene/level_system_render_extract.rs",
    ] {
        assert_source_excludes_file(
            &manifest_root.join(relative),
            &["RenderFrameExtract::from_snapshot"],
            "scene production extraction must populate RenderFrameExtract directly; snapshot adapters are allowed only for preview/test/roundtrip/synthetic helpers",
        );
    }

    let submit_root = manifest_root
        .join("src")
        .join("graphics")
        .join("runtime")
        .join("render_framework")
        .join("submit_frame_extract");
    assert_runtime_submit_tree_excludes_snapshot_adapters(&submit_root);
}

fn spawn_camera_on_layer(world: &mut World, layer_mask: u32) -> crate::scene::EntityId {
    let camera = world.spawn_node(NodeKind::Camera);
    world
        .insert(
            camera,
            CameraComponent {
                projection_mode: crate::core::framework::render::ProjectionMode::Perspective,
                ..CameraComponent::default()
            },
        )
        .unwrap();
    world.set_render_layer_mask(camera, layer_mask).unwrap();
    camera
}

fn spawn_mesh_on_layer(
    world: &mut World,
    layer_mask: u32,
    mobility: Mobility,
) -> crate::scene::EntityId {
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .insert(
            entity,
            MeshRenderer::from_handles(
                model_handle(&format!("res://models/direct-{entity}.obj")),
                material_handle(&format!("res://materials/direct-{entity}.zmaterial")),
            ),
        )
        .unwrap();
    world.set_render_layer_mask(entity, layer_mask).unwrap();
    if mobility != Mobility::Dynamic {
        world.set_mobility(entity, mobility).unwrap();
    }
    entity
}

fn spawn_sprite_on_layer(world: &mut World, layer_mask: u32) -> crate::scene::EntityId {
    let entity = world.spawn_node(NodeKind::Mesh);
    let _ = world.remove::<MeshRenderer>(entity).unwrap();
    world
        .insert(
            entity,
            Sprite2dComponent {
                image: ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
                    &format!("res://textures/sprite-{entity}.png"),
                )),
                material: Some(material_handle(&format!(
                    "res://materials/sprite-{entity}.zmaterial"
                ))),
                custom_size: Some(Vec2::new(2.0, 3.0)),
                color: Vec4::new(1.0, 0.5, 0.25, 0.75),
                z_order: 5,
                material_alpha_mode: RenderMaterialAlphaMode::Blend,
                ..Sprite2dComponent::default()
            },
        )
        .unwrap();
    world.set_render_layer_mask(entity, layer_mask).unwrap();
    entity
}

fn spawn_light_on_layer(
    world: &mut World,
    kind: NodeKind,
    layer_mask: u32,
) -> crate::scene::EntityId {
    let entity = world.spawn_node(kind);
    world.set_render_layer_mask(entity, layer_mask).unwrap();
    entity
}

fn spawn_post_process_volume_on_layer(
    world: &mut World,
    layer_mask: u32,
    volume: PostProcessVolumeComponent,
) -> crate::scene::EntityId {
    let entity = world.spawn_node(NodeKind::Mesh);
    let _ = world.remove::<MeshRenderer>(entity).unwrap();
    world.insert(entity, volume).unwrap();
    world.set_render_layer_mask(entity, layer_mask).unwrap();
    entity
}

fn test_pose(bone: &str) -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: Some("Locomotion".to_string()),
        bones: vec![AnimationPoseBone {
            name: bone.to_string(),
            local_transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        }],
    }
}

fn assert_runtime_submit_tree_excludes_snapshot_adapters(root: &std::path::Path) {
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            assert_runtime_submit_tree_excludes_snapshot_adapters(&path);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let production_source = source.split("#[cfg(test)]").next().unwrap_or(&source);
        assert_source_excludes(
            &path,
            production_source,
            &["RenderFrameExtract::from_snapshot", "ViewportRenderFrame::from_snapshot"],
            "runtime submit_frame_extract production code must consume RenderFrameExtract through ViewportRenderFrame::from_extract; snapshot adapters are limited to tests/preview/synthetic validation",
        );
    }
}

fn assert_source_excludes_file(path: &std::path::Path, forbidden: &[&str], message: &str) {
    let source = std::fs::read_to_string(path).unwrap();
    assert_source_excludes(path, &source, forbidden, message);
}

fn assert_source_excludes(path: &std::path::Path, source: &str, forbidden: &[&str], message: &str) {
    for token in forbidden {
        assert!(
            !source.contains(token),
            "{message}: found `{token}` in {path:?}"
        );
    }
}
