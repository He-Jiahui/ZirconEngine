use crate::core::framework::render::{
    RenderExtractContext, RenderWorldSnapshotHandle, SceneViewportExtractRequest,
};
use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::{MeshRenderer, Mobility, NodeRecord};
use crate::scene::{NodeKind, SystemStage, World};

const LARGE_HIERARCHY_NODE_COUNT: usize = 256;

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
    assert!(static_reparent_error.contains("Static"));
    assert!(!world.has_pending_scene_systems());

    assert!(!world.has_pending_scene_systems());
}

#[test]
fn imported_records_validate_missing_parents_and_preserve_out_of_order_links() {
    let mut missing_parent_record = detached_node_record(10, NodeKind::Mesh);
    missing_parent_record.parent = Some(999);
    let mut world = World::empty();
    world.insert_node_record(missing_parent_record).unwrap();
    assert_eq!(world.node_record(10).unwrap().parent, Some(999));

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.node_record(10).unwrap().parent, None);

    let mut parent_record = detached_node_record(42, NodeKind::Cube);
    parent_record.transform = Transform::from_translation(Vec3::new(3.0, 0.0, 0.0));
    let mut child_record = detached_node_record(43, NodeKind::Mesh);
    child_record.parent = Some(parent_record.id);
    child_record.transform = Transform::from_translation(Vec3::new(4.0, 0.0, 0.0));

    let mut world = World::empty();
    world.insert_node_record(child_record).unwrap();
    world.insert_node_record(parent_record).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);

    assert_eq!(world.node_record(43).unwrap().parent, Some(42));
    assert_eq!(
        world.world_transform(43).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
}

#[test]
fn hierarchy_cycle_rejection_preserves_existing_parent_state() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(parent)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());

    let error = world.set_parent_checked(parent, Some(child)).unwrap_err();

    assert!(error.contains("cycle"));
    assert_eq!(world.find_node(parent).unwrap().parent, None);
    assert_eq!(world.find_node(child).unwrap().parent, Some(parent));
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn active_hierarchy_propagates_inactive_and_reactivated_ancestors() {
    let mut world = World::new();
    let root = world.spawn_node(NodeKind::Cube);
    let middle = world.spawn_node(NodeKind::Cube);
    let leaf = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(middle, Some(root)).unwrap();
    world.set_parent_checked(leaf, Some(middle)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    world.set_active_self(root, false).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.active_in_hierarchy(middle), Some(false));
    assert_eq!(world.active_in_hierarchy(leaf), Some(false));
    assert!(world
        .build_prepared_render_frame_extract(&RenderExtractContext::new(
            RenderWorldSnapshotHandle::new(201),
            SceneViewportExtractRequest::default(),
        ))
        .geometry
        .meshes
        .iter()
        .all(|mesh| mesh.node_id != leaf));

    world.set_active_self(root, true).unwrap();
    world.set_active_self(middle, false).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.active_in_hierarchy(root), Some(true));
    assert_eq!(world.active_in_hierarchy(middle), Some(false));
    assert_eq!(world.active_in_hierarchy(leaf), Some(false));

    world.set_active_self(middle, true).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.active_in_hierarchy(leaf), Some(true));
    assert!(world
        .build_prepared_render_frame_extract(&RenderExtractContext::new(
            RenderWorldSnapshotHandle::new(202),
            SceneViewportExtractRequest::default(),
        ))
        .geometry
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == leaf));
}

#[test]
fn post_update_propagates_large_hierarchy_transform_and_active_state() {
    let mut world = World::new();
    let mut entities = Vec::with_capacity(LARGE_HIERARCHY_NODE_COUNT);
    for index in 0..LARGE_HIERARCHY_NODE_COUNT {
        let entity = world.spawn_node(if index + 1 == LARGE_HIERARCHY_NODE_COUNT {
            NodeKind::Mesh
        } else {
            NodeKind::Cube
        });
        world
            .update_transform(
                entity,
                Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            )
            .unwrap();
        if let Some(parent) = entities.last().copied() {
            world.set_parent_checked(entity, Some(parent)).unwrap();
        }
        entities.push(entity);
    }
    let hidden_ancestor = entities[LARGE_HIERARCHY_NODE_COUNT / 2];
    let deepest = *entities.last().unwrap();
    world.set_active_self(hidden_ancestor, false).unwrap();

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);

    assert_eq!(
        world.world_transform(deepest).unwrap().translation,
        Vec3::new(LARGE_HIERARCHY_NODE_COUNT as f32, 0.0, 0.0)
    );
    assert_eq!(world.active_in_hierarchy(deepest), Some(false));
    assert!(world
        .nodes()
        .iter()
        .find(|node| node.id == deepest)
        .is_some_and(|node| node.parent == Some(entities[LARGE_HIERARCHY_NODE_COUNT - 2])));
}

#[test]
fn mobility_changes_refresh_visibility_buckets_without_transform_rebuild() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let mesh = world.spawn_node(NodeKind::Mesh);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert!(world.set_mobility(mesh, Mobility::Static).unwrap());
    assert!(world.update_transform(mesh, Transform::default()).is_err());
    assert!(world.set_parent_checked(mesh, Some(parent)).is_err());
    let static_extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(301),
        SceneViewportExtractRequest::default(),
    ));
    assert!(static_extract.visibility.static_entities.contains(&mesh));
    assert!(!static_extract.visibility.dynamic_entities.contains(&mesh));

    assert!(world.set_mobility(mesh, Mobility::Dynamic).unwrap());
    let dynamic_extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(302),
        SceneViewportExtractRequest::default(),
    ));
    assert!(dynamic_extract.visibility.dynamic_entities.contains(&mesh));
    assert!(!dynamic_extract.visibility.static_entities.contains(&mesh));
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

fn detached_node_record(id: u64, kind: NodeKind) -> NodeRecord {
    let mut source = World::empty();
    let entity = source.spawn_node(kind);
    let mut record = source.node_record(entity).unwrap();
    record.id = id;
    record.name = format!("Imported {id}");
    record
}

fn pending_reparented_world() -> World {
    let mut world = World::new();
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
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(12.0, 0.0, 0.0)
    );
    world
}
