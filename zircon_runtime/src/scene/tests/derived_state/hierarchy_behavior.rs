use super::*;
use crate::scene::components::{ActiveInHierarchy, WorldMatrix};

#[test]
fn imported_records_validate_missing_parents_and_preserve_out_of_order_links() {
    let mut missing_parent_record = detached_node_record(10, NodeKind::Mesh);
    missing_parent_record.parent = Some(999);
    let mut world = World::empty();
    world.insert_node_record(missing_parent_record).unwrap();
    assert_eq!(world.node_record(10).unwrap().parent, Some(999));

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.node_record(10).unwrap().parent, None);
    assert_eq!(
        world
            .get::<WorldMatrix>(10)
            .map(|matrix| matrix.0.to_scale_rotation_translation().2),
        Some(Vec3::ZERO),
        "validity repair must also publish the orphan as a hierarchy root"
    );
    assert_eq!(
        world.get::<ActiveInHierarchy>(10).map(|active| active.0),
        Some(true),
        "validity repair must propagate active state through the repaired root index"
    );

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
fn componentless_root_updates_the_hierarchy_index_before_derived_propagation() {
    let mut world = World::empty();
    assert!(world.spawn_empty_at(70).unwrap());

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);

    assert_eq!(
        world
            .get::<WorldMatrix>(70)
            .map(|matrix| matrix.0.to_scale_rotation_translation().2),
        Some(Vec3::ZERO)
    );
    assert_eq!(
        world.get::<ActiveInHierarchy>(70).map(|active| active.0),
        Some(true)
    );
    let detached = world
        .remove_entity_recursive(70)
        .expect("componentless root should remain addressable through the hierarchy transaction");
    assert_eq!(detached.entity_ids().collect::<Vec<_>>(), vec![70]);
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

    assert!(error.to_string().contains("cycle"));
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
    assert!(
        world
            .build_prepared_render_frame_extract(&RenderExtractContext::new(
                RenderWorldSnapshotHandle::new(201),
                SceneViewportExtractRequest::default(),
            ))
            .geometry
            .meshes
            .iter()
            .all(|mesh| mesh.node_id != leaf)
    );

    world.set_active_self(root, true).unwrap();
    world.set_active_self(middle, false).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.active_in_hierarchy(root), Some(true));
    assert_eq!(world.active_in_hierarchy(middle), Some(false));
    assert_eq!(world.active_in_hierarchy(leaf), Some(false));

    world.set_active_self(middle, true).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.active_in_hierarchy(leaf), Some(true));
    assert!(
        world
            .build_prepared_render_frame_extract(&RenderExtractContext::new(
                RenderWorldSnapshotHandle::new(202),
                SceneViewportExtractRequest::default(),
            ))
            .geometry
            .meshes
            .iter()
            .any(|mesh| mesh.node_id == leaf)
    );
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
    assert!(
        world
            .nodes()
            .iter()
            .find(|node| node.id == deepest)
            .is_some_and(|node| node.parent == Some(entities[LARGE_HIERARCHY_NODE_COUNT - 2]))
    );
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
