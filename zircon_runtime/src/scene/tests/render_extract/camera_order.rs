use super::*;

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
    assert_eq!(
        extract
            .view
            .cameras
            .iter()
            .map(|camera| camera.entity)
            .collect::<Vec<_>>(),
        vec![Some(texture_camera), Some(primary_a), Some(primary_b)]
    );
    let texture_descriptor = extract
        .view
        .cameras
        .iter()
        .find(|camera| camera.entity == Some(texture_camera))
        .expect("scene-backed extract should carry texture target descriptor");
    assert_eq!(texture_descriptor.render_type, CameraRenderType::Base);
    assert!(matches!(
        texture_descriptor.target,
        RenderCameraTarget::Texture(_)
    ));
    assert_eq!(
        texture_descriptor.culling_mask.to_legacy_mask_lossy(),
        0b0010
    );
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
    let texture_report_camera = report
        .cameras
        .iter()
        .find(|camera| camera.entity == texture_camera)
        .expect("texture target camera should keep its camera payload");
    assert!(matches!(
        texture_report_camera.camera.target,
        RenderCameraTarget::Texture(_)
    ));
    assert!(texture_report_camera.hdr);
    assert_eq!(
        texture_report_camera
            .camera
            .culling_mask
            .to_legacy_mask_lossy(),
        0b0010
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
            camera: Some({
                let mut camera = camera_descriptor_with_layers(0b0100);
                camera.render_order = 42;
                camera
            }),
            ..SceneViewportExtractRequest::default()
        },
    ));

    assert_eq!(extract.view.scene_camera_entity, None);
    assert!(extract.view.scene_camera_order_report.is_none());
    assert_eq!(extract.view.cameras.len(), 1);
    assert_eq!(extract.view.cameras[0].entity, None);
    assert_eq!(extract.view.cameras[0].render_order, 42);
    assert_eq!(
        extract.view.cameras[0].culling_mask.to_legacy_mask_lossy(),
        0b0100
    );
}

#[test]
fn render_frame_extract_keeps_custom_target_layer_geometry_for_visibility_views() {
    let mut world = World::empty();
    let primary = spawn_camera_on_layer(&mut world, 0b0001);
    world.set_active_camera(primary);

    let texture_camera = spawn_camera_on_layer(&mut world, 0b0010);
    {
        let component = world.get_mut::<CameraComponent>(texture_camera).unwrap();
        component.order = -1;
        component.target = RenderCameraTarget::Texture(ResourceHandle::<TextureMarker>::new(
            ResourceId::from_stable_label("res://textures/custom-target-visibility.png"),
        ));
    }

    let main_mesh = spawn_mesh_on_layer(&mut world, 0b0001, Mobility::Static);
    let custom_target_mesh = spawn_mesh_on_layer(&mut world, 0b0010, Mobility::Static);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(708),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.view.scene_camera_entity, Some(primary));
    assert!(extract
        .geometry
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == main_mesh));
    assert!(extract
        .geometry
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == custom_target_mesh));
    assert_eq!(
        extract.view.selected_camera_layers().to_legacy_mask_lossy(),
        0b0001,
        "main camera layer remains unchanged; the layer union is only an extract candidate set"
    );
}
