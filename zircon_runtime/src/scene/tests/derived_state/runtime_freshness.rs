use super::*;

#[test]
fn projected_reads_stay_fresh_until_post_update_refreshes_retained_cache() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());

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

    assert!(world.has_pending_scene_systems());
    assert!(world
        .nodes()
        .iter()
        .find(|node| node.id == child)
        .is_some_and(|node| node.parent.is_none()));
    assert!(world
        .node_records()
        .iter()
        .find(|node| node.id == child)
        .is_some_and(|node| node.parent == Some(parent)));
    assert_eq!(world.find_node(child).unwrap().parent, Some(parent));
    assert_eq!(world.active_in_hierarchy(child), Some(false));
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert_eq!(
        world
            .world_matrix(child)
            .unwrap()
            .to_scale_rotation_translation()
            .2,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert!(world.has_pending_scene_systems());

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    let refreshed_nodes = world.nodes().to_vec();
    assert!(refreshed_nodes
        .iter()
        .find(|node| node.id == child)
        .is_some_and(|node| node.parent == Some(parent)));
    assert_eq!(world.active_in_hierarchy(child), Some(false));
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert!(world.has_pending_scene_systems());

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.nodes(), refreshed_nodes.as_slice());
    assert!(world.has_pending_scene_systems());

    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn no_op_mutators_do_not_mark_derived_state_dirty() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    let static_child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(parent)).unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)))
        .unwrap();
    world.set_active_self(parent, false).unwrap();
    world.set_render_layer_mask(child, 0b0010).unwrap();
    world.set_mobility(static_child, Mobility::Static).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());

    assert!(!world.set_parent_checked(child, Some(parent)).unwrap());
    assert!(!world
        .update_transform(child, Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)))
        .unwrap());
    assert!(!world.set_active_self(parent, false).unwrap());
    assert!(!world.set_render_layer_mask(child, 0b0010).unwrap());
    assert!(!world.set_mobility(static_child, Mobility::Static).unwrap());

    assert!(!world.has_pending_scene_systems());
    let static_reparent_error = world.set_parent_checked(static_child, None).unwrap_err();
    assert!(static_reparent_error.to_string().contains("Static"));
    assert!(!world.has_pending_scene_systems());

    assert!(!world.has_pending_scene_systems());
}

#[test]
fn render_extract_prepare_flushes_direct_frame_and_legacy_viewport_paths() {
    let mut world = pending_reparented_world();
    let child = world
        .node_records()
        .into_iter()
        .find(|node| matches!(node.kind, NodeKind::Mesh))
        .unwrap()
        .id;
    assert!(world.has_pending_scene_systems());

    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest::default());
    assert!(packet.scene.meshes.iter().all(|mesh| mesh.node_id != child));
    assert!(world.has_pending_scene_systems());

    let frame = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(401),
        SceneViewportExtractRequest::default(),
    ));
    assert!(frame
        .geometry
        .meshes
        .iter()
        .all(|mesh| mesh.node_id != child));
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn property_path_node_cache_changes_mark_dirty_and_zero_morph_extension_is_not_noop() {
    let mut world = World::new();
    let mesh = world.spawn_node(NodeKind::Mesh);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());

    let tint_path = ComponentPropertyPath::parse("MeshRenderer.tint").unwrap();
    assert!(world
        .set_property(
            mesh,
            &tint_path,
            ScenePropertyValue::Vec4([0.25, 0.5, 0.75, 1.0]),
        )
        .unwrap());
    assert!(world.has_pending_scene_systems());
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    let morph_path = ComponentPropertyPath::parse("MeshRenderer.morph_weights.2").unwrap();
    assert!(world
        .set_property(mesh, &morph_path, ScenePropertyValue::Scalar(0.0))
        .unwrap());
    assert_eq!(
        world.get::<MeshRenderer>(mesh).unwrap().morph_weights,
        vec![0.0; 3]
    );
    assert!(world.has_pending_scene_systems());
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert!(!world
        .set_property(mesh, &morph_path, ScenePropertyValue::Scalar(0.0))
        .unwrap());
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn active_camera_selection_marks_render_extract_freshness_without_rebuilding_scheduler() {
    let mut world = World::new();
    let original_camera = world.active_camera();
    let second_camera = world.spawn_node(NodeKind::Camera);
    world
        .update_transform(
            second_camera,
            Transform::from_translation(Vec3::new(11.0, 0.0, 0.0)),
        )
        .unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());

    world.set_active_camera(original_camera);
    assert!(!world.has_pending_scene_systems());
    world.set_active_camera(second_camera);
    assert!(world.has_pending_scene_systems());

    let frame = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(501),
        SceneViewportExtractRequest::default(),
    ));
    assert_eq!(
        frame.view.camera.transform.translation,
        Vec3::new(11.0, 0.0, 0.0)
    );
    assert!(!world.has_pending_scene_systems());
}
