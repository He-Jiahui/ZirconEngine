use super::*;

#[test]
fn world_mutations_mark_derived_state_dirty_until_post_update_systems_flush() {
    let mut world = crate::scene::World::new();
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
    world.set_active_self(parent, false).unwrap();

    assert!(world.nodes().iter().all(|node| node.id != parent));
    assert!(world.node_records().iter().any(|node| node.id == parent));

    assert!(world.has_pending_scene_systems());
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert_eq!(world.active_in_hierarchy(child), Some(false));
    assert!(world.has_pending_scene_systems());

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);

    assert!(world.has_pending_scene_systems());
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert_eq!(world.active_in_hierarchy(child), Some(false));

    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert!(!world.has_pending_scene_systems());
}

#[test]
fn render_extract_prepare_flushes_parent_reorder_and_active_changes() {
    let mut world = crate::scene::World::new();
    let first_parent = world.spawn_node(NodeKind::Cube);
    let second_parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);

    world
        .update_transform(
            first_parent,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(
            second_parent,
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    world.set_parent_checked(child, Some(first_parent)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    world
        .set_parent_checked(child, Some(second_parent))
        .unwrap();
    world.set_active_self(second_parent, false).unwrap();

    assert!(
        world
            .nodes()
            .iter()
            .find(|node| node.id == child)
            .is_some_and(|node| node.parent == Some(first_parent))
    );
    assert!(
        world
            .node_records()
            .iter()
            .find(|node| node.id == child)
            .is_some_and(|node| node.parent == Some(second_parent))
    );
    assert_eq!(world.active_in_hierarchy(child), Some(false));
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(12.0, 0.0, 0.0)
    );
    assert!(world.has_pending_scene_systems());
    assert!(
        world
            .to_render_extract()
            .scene
            .meshes
            .iter()
            .all(|mesh| mesh.node_id != child)
    );
    assert!(world.has_pending_scene_systems());

    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert!(!world.has_pending_scene_systems());
}

#[test]
fn canonical_render_frame_extract_populates_scene_sections_directly() {
    let mut world = crate::scene::World::new();
    let camera = world.active_camera();
    let mesh = world.spawn_node(NodeKind::Mesh);
    world.set_render_layer_mask(camera, 0b1010).unwrap();
    world.set_render_layer_mask(mesh, 0b1010).unwrap();
    world
        .update_transform(mesh, Transform::from_translation(Vec3::new(4.0, 5.0, 6.0)))
        .unwrap();
    world.set_mobility(mesh, Mobility::Static).unwrap();
    let debug = RenderVirtualGeometryDebugState {
        forced_mip: Some(3),
        visualize_bvh: true,
        ..RenderVirtualGeometryDebugState::default()
    };
    let context = RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(55),
        SceneViewportExtractRequest {
            settings: ViewportRenderSettings {
                display_mode: DisplayMode::WireOnly,
                preview_lighting: false,
                preview_skybox: false,
                ..ViewportRenderSettings::default()
            },
            active_camera_override: None,
            camera: None,
            viewport_size: Some(UVec2::new(1280, 720)),
            virtual_geometry_debug: Some(debug),
        },
    );

    let extract = world.build_prepared_render_frame_extract(&context);

    assert_eq!(extract.world.raw(), 55);
    assert_eq!(extract.view.camera.aspect_ratio, 1280.0 / 720.0);
    assert!(extract.geometry.meshes.iter().any(|snapshot| {
        snapshot.node_id == mesh
            && snapshot.transform.translation == Vec3::new(4.0, 5.0, 6.0)
            && snapshot.mobility == Mobility::Static
            && snapshot.common.is_static
            && snapshot.common.layer_mask.to_scene_schema_v1_mask_lossy() == 0b1010
    }));
    assert_eq!(extract.geometry.virtual_geometry_debug, Some(debug));
    assert!(extract.geometry.virtual_geometry.is_some());
    assert!(
        extract
            .lighting
            .hybrid_global_illumination
            .as_ref()
            .is_some_and(|hybrid_gi| !hybrid_gi.enabled)
    );
    assert_eq!(extract.post_process.display_mode, DisplayMode::WireOnly);
    assert!(!extract.post_process.preview.lighting_enabled);
    assert!(!extract.post_process.preview.skybox_enabled);
    assert_eq!(
        extract.visibility.renderables.len(),
        extract.geometry.meshes.len()
    );
    assert!(
        extract
            .visibility
            .static_entities
            .iter()
            .any(|entity| *entity == mesh)
    );
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn prepared_render_frame_extract_queues_meshes_from_mesh_renderer_alpha_hints() {
    let mut world = crate::scene::World::new();
    let alpha_mask_mesh = world.spawn_node(NodeKind::Mesh);
    let transparent_mesh = world.spawn_node(NodeKind::Mesh);
    world
        .get_mut::<MeshRenderer>(alpha_mask_mesh)
        .unwrap()
        .material_alpha_mode = RenderMaterialAlphaMode::Mask { cutoff: 0.37 };
    world
        .get_mut::<MeshRenderer>(transparent_mesh)
        .unwrap()
        .material_alpha_mode = RenderMaterialAlphaMode::Blend;
    world
        .update_transform(
            transparent_mesh,
            Transform::from_translation(Vec3::new(0.0, 0.0, 9.0)),
        )
        .unwrap();
    let context = RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(56),
        SceneViewportExtractRequest::default(),
    );

    let extract = world.build_prepared_render_frame_extract(&context);
    let alpha_mask_index = extract
        .geometry
        .meshes
        .iter()
        .position(|mesh| mesh.node_id == alpha_mask_mesh)
        .expect("alpha-mask mesh should be extracted");
    let transparent_index = extract
        .geometry
        .meshes
        .iter()
        .position(|mesh| mesh.node_id == transparent_mesh)
        .expect("transparent mesh should be extracted");

    assert!(extract.geometry.phase_inputs.iter().any(|input| {
        input.entity == alpha_mask_mesh
            && input.mesh_index == alpha_mask_index
            && input.material_alpha_mode == RenderMaterialAlphaMode::Mask { cutoff: 0.37 }
    }));
    assert!(extract.geometry.phase_inputs.iter().any(|input| {
        input.entity == transparent_mesh
            && input.mesh_index == transparent_index
            && input.material_alpha_mode == RenderMaterialAlphaMode::Blend
            && input.depth == 9.0
    }));
    assert!(
        extract
            .geometry
            .phase_queue
            .items_for_phase(RenderPhase::AlphaMask3d)
            .any(|item| item.entity == alpha_mask_mesh)
    );
    assert!(
        extract
            .geometry
            .phase_queue
            .items_for_phase(RenderPhase::Transparent3d)
            .any(|item| item.entity == transparent_mesh)
    );
}

#[test]
fn render_extract_filters_meshes_by_active_camera_layers() {
    let mut world = crate::scene::World::new();
    let camera = world.active_camera();
    let visible_mesh = world.spawn_node(NodeKind::Mesh);
    let hidden_mesh = world.spawn_node(NodeKind::Mesh);
    world.set_render_layer_mask(camera, 0b0010).unwrap();
    world.set_render_layer_mask(visible_mesh, 0b0010).unwrap();
    world.set_render_layer_mask(hidden_mesh, 0b0100).unwrap();

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(57),
        SceneViewportExtractRequest::default(),
    ));

    assert!(
        extract
            .geometry
            .meshes
            .iter()
            .any(|mesh| mesh.node_id == visible_mesh)
    );
    assert!(
        extract
            .geometry
            .meshes
            .iter()
            .all(|mesh| mesh.node_id != hidden_mesh)
    );
    assert!(
        extract.geometry.meshes.iter().all(|mesh| mesh
            .common
            .layer_mask
            .to_scene_schema_v1_mask_lossy()
            & 0b0010
            != 0)
    );
    assert!(
        extract
            .view
            .selected_camera_layers()
            .intersects_scene_schema_v1_mask(0b0010)
    );
}

#[test]
fn explicit_render_camera_snapshot_layers_override_scene_camera_layers() {
    let mut world = crate::scene::World::new();
    let camera = world.active_camera();
    let visible_mesh = world.spawn_node(NodeKind::Mesh);
    let hidden_mesh = world.spawn_node(NodeKind::Mesh);
    world.set_render_layer_mask(camera, 0b0010).unwrap();
    world.set_render_layer_mask(visible_mesh, 0b0100).unwrap();
    world.set_render_layer_mask(hidden_mesh, 0b0010).unwrap();

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(58),
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
            .any(|mesh| mesh.node_id == visible_mesh)
    );
    assert!(
        extract
            .geometry
            .meshes
            .iter()
            .all(|mesh| mesh.node_id != hidden_mesh)
    );
    assert!(
        extract
            .view
            .selected_camera_layers()
            .intersects_scene_schema_v1_mask(0b0100)
    );
}

#[test]
fn render_extract_projects_scene_camera_component_product_fields() {
    let mut world = crate::scene::World::new();
    let camera = world.active_camera();
    *world.get_mut::<CameraComponent>(camera).unwrap() = CameraComponent {
        projection_mode: ProjectionMode::Orthographic,
        fov_y_radians: 0.85,
        ortho_size: 14.0,
        z_near: 0.05,
        z_far: 750.0,
        viewport: Some(RenderViewportRect::new(
            UVec2::new(16, 32),
            UVec2::new(400, 200),
        )),
        order: 4,
        is_active: false,
        hdr: true,
        exposure_ev100: 12.0,
        clear_color: RenderCameraClearColor::None,
        msaa_samples: 4,
        ..CameraComponent::default()
    };

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(59),
        SceneViewportExtractRequest {
            viewport_size: Some(UVec2::new(1280, 720)),
            ..SceneViewportExtractRequest::default()
        },
    ));

    assert_eq!(
        extract.view.camera.projection_mode,
        ProjectionMode::Orthographic
    );
    assert_eq!(extract.view.camera.fov_y_radians, 0.85);
    assert_eq!(extract.view.camera.ortho_size, 14.0);
    assert_eq!(extract.view.camera.z_near, 0.05);
    assert_eq!(extract.view.camera.z_far, 750.0);
    assert_eq!(extract.view.camera.aspect_ratio, 2.0);
    assert_eq!(
        extract
            .view
            .selected_camera_descriptor()
            .expect("scene camera descriptor should be selected")
            .render_order,
        4
    );
    assert!(!extract.view.camera.is_active);
    assert!(extract.view.camera.hdr);
    assert_eq!(extract.view.camera.exposure_ev100, 12.0);
    assert_eq!(
        extract.post_process.exposure.mode,
        RenderExposureMode::Manual
    );
    assert_eq!(extract.post_process.exposure.manual_ev100, 12.0);
    assert_eq!(
        extract
            .view
            .selected_camera_descriptor()
            .expect("scene camera descriptor should be selected")
            .clear,
        RenderCameraClear::None
    );
    assert_eq!(extract.view.camera.msaa_samples, 4);
}

fn camera_descriptor_with_layers(mask: u32) -> CameraRenderDescriptor {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    camera.culling_mask = RenderLayerSet::from_scene_schema_v1_mask(mask);
    camera.volume_mask = camera.culling_mask.clone();
    camera
}

#[test]
fn inactive_render_camera_extracts_no_scene_renderables() {
    let mut world = crate::scene::World::new();
    let camera = world.active_camera();
    world.get_mut::<CameraComponent>(camera).unwrap().is_active = false;
    world.spawn_node(NodeKind::Mesh);
    world.spawn_node(NodeKind::DirectionalLight);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(60),
        SceneViewportExtractRequest::default(),
    ));

    assert!(!extract.view.camera.is_active);
    assert!(extract.geometry.meshes.is_empty());
    assert!(extract.geometry.phase_inputs.is_empty());
    assert!(extract.visibility.renderable_entities.is_empty());
    assert!(extract.visibility.renderables.is_empty());
    assert!(extract.lighting.directional_lights.is_empty());

    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest::default());
    assert!(!packet.scene.camera.is_active);
    assert!(packet.scene.meshes.is_empty());
    assert!(packet.scene.directional_lights.is_empty());
}

#[test]
fn mobility_changes_are_node_cache_dirty_without_transform_flush() {
    let mut world = crate::scene::World::new();
    let entity = world.spawn_node(NodeKind::Mesh);

    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());
    assert!(world.set_mobility(entity, Mobility::Static).unwrap());

    assert!(world.has_pending_scene_systems());
    assert_eq!(world.world_transform(entity).unwrap(), Transform::default());
    assert!(world.has_pending_scene_systems());
}
