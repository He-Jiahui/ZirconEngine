use super::*;

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
        *extract.view.selected_camera_layers(),
        RenderLayerSet::from_scene_schema_v1_mask(0b0111)
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
    assert_eq!(
        dynamic_row
            .common
            .layer_mask
            .to_scene_schema_v1_mask_lossy(),
        0b0010
    );

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
    assert_eq!(
        extract.sprites.sprites[0]
            .common
            .layer_mask
            .to_scene_schema_v1_mask_lossy(),
        0b0010
    );
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
        .output_transfer_node
        .as_deref()
        .is_some_and(|node| node == "output-transfer"));
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
fn render_frame_extract_selects_mesh_lod_by_camera_distance() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0001);
    world.set_active_camera(camera);
    world
        .update_transform(camera, Transform::default())
        .expect("test camera transform should be mutable");
    let mesh_entity = spawn_mesh_on_layer(&mut world, 0b0001, Mobility::Dynamic);
    let base_model = model_handle(&format!("res://models/direct-{mesh_entity}.obj"));
    let base_material = material_handle(&format!("res://materials/direct-{mesh_entity}.zmaterial"));
    let lod_model = model_handle("res://models/direct-lod1.obj");
    let lod_mesh = mesh_handle("res://meshes/direct-lod1.zmesh");
    let lod_material = material_handle("res://materials/direct-lod1.zmaterial");

    world.get_mut::<MeshRenderer>(mesh_entity).unwrap().lods = vec![MeshRendererLodLevel {
        min_distance: 10.0,
        model: lod_model,
        mesh: Some(lod_mesh),
        material: lod_material,
        primitives: Vec::new(),
    }];

    world
        .update_transform(
            mesh_entity,
            Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
        )
        .unwrap();
    let near_extract = world.to_render_frame_extract();
    let near_mesh = near_extract
        .geometry
        .meshes
        .iter()
        .find(|mesh| mesh.node_id == mesh_entity)
        .expect("near mesh should be extracted");
    assert_eq!(near_mesh.model, base_model);
    assert_eq!(near_mesh.mesh, None);
    assert_eq!(near_mesh.material, base_material);
    assert!(near_mesh.mesh_lod.is_none());

    world
        .update_transform(
            mesh_entity,
            Transform::from_translation(Vec3::new(0.0, 0.0, 12.0)),
        )
        .unwrap();
    let far_extract = world.to_render_frame_extract();
    let far_mesh = far_extract
        .geometry
        .meshes
        .iter()
        .find(|mesh| mesh.node_id == mesh_entity)
        .expect("far mesh should be extracted");
    assert_eq!(far_mesh.model, lod_model);
    assert_eq!(far_mesh.mesh, Some(lod_mesh));
    assert_eq!(far_mesh.material, lod_material);
    let far_mesh_lod = far_mesh
        .mesh_lod
        .expect("far mesh should carry lod metadata");
    assert_eq!(far_mesh_lod.level_index, 0);
    assert_eq!(far_mesh_lod.min_distance, 10.0);
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
        .all(|renderable| renderable
            .render_layer_mask
            .intersects(&RenderLayerSet::from_scene_schema_v1_mask(0b0010))));
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
