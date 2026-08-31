use super::*;
use crate::scene::components::{ActiveInHierarchy, Hierarchy, WorldMatrix};

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
    let parent = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
fn raw_hierarchy_cycle_repair_republishes_all_corrected_derived_rows() {
    let mut world = World::new();
    let first = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let second = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .update_transform(first, Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)))
        .unwrap();
    world
        .update_transform(
            second,
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        )
        .unwrap();
    assert!(world.set_active_self(second, false).unwrap());
    world.set_parent_checked(first, Some(second)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert_eq!(
        world.world_transform(first).unwrap().translation,
        Vec3::new(11.0, 0.0, 0.0)
    );

    world.reset_ecs_frame_performance_diagnostics();
    world.get_mut::<Hierarchy>(second).unwrap().parent = Some(first);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert_eq!(world.find_node(first).unwrap().parent, None);
    assert_eq!(world.find_node(second).unwrap().parent, Some(first));
    assert_eq!(
        world.world_transform(first).unwrap().translation,
        Vec3::new(1.0, 0.0, 0.0)
    );
    assert_eq!(
        world.world_transform(second).unwrap().translation,
        Vec3::new(11.0, 0.0, 0.0)
    );
    assert_eq!(world.active_in_hierarchy(first), Some(true));
    assert_eq!(world.active_in_hierarchy(second), Some(false));
    assert_eq!(
        world
            .nodes()
            .iter()
            .find(|node| node.id == first)
            .unwrap()
            .parent,
        None
    );
    assert_eq!(
        world
            .nodes()
            .iter()
            .find(|node| node.id == second)
            .unwrap()
            .parent,
        Some(first)
    );

    let diagnostics = world.ecs_frame_performance_diagnostics().derived_state;
    assert_eq!(diagnostics.active_propagation_entities, 2);
    assert_eq!(diagnostics.world_matrix_propagation_entities, 2);
    assert_eq!(diagnostics.node_cache_rebuilt_entities, 2);
}

#[test]
fn raw_three_node_cycle_repair_breaks_the_first_stable_edge_only() {
    let mut world = World::new();
    let first = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let second = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let third = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world.set_parent_checked(first, Some(second)).unwrap();
    world.set_parent_checked(second, Some(third)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    world.get_mut::<Hierarchy>(third).unwrap().parent = Some(first);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert_eq!(world.find_node(first).unwrap().parent, None);
    assert_eq!(world.find_node(second).unwrap().parent, Some(third));
    assert_eq!(world.find_node(third).unwrap().parent, Some(first));
}

#[test]
fn raw_self_cycle_repair_preserves_an_earlier_descendant_attachment() {
    let mut world = World::empty();
    let descendant = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let cycle_owner = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    world
        .update_transform(
            descendant,
            Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(
            cycle_owner,
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_parent_checked(descendant, Some(cycle_owner))
        .unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    world.get_mut::<Hierarchy>(cycle_owner).unwrap().parent = Some(cycle_owner);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert_eq!(world.find_node(cycle_owner).unwrap().parent, None);
    assert_eq!(
        world.find_node(descendant).unwrap().parent,
        Some(cycle_owner)
    );
    assert_eq!(
        world.world_transform(descendant).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert_eq!(
        world
            .nodes()
            .iter()
            .find(|node| node.id == descendant)
            .unwrap()
            .parent,
        Some(cycle_owner)
    );
}

#[test]
fn deserialized_world_rebuilds_node_cache_before_incremental_reparent_projection() {
    let mut world = World::new();
    let parent = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .update_transform(
            parent,
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)))
        .unwrap();
    assert!(world.set_active_self(parent, false).unwrap());
    world.set_parent_checked(child, Some(parent)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    let serialized = serde_json::to_string(&world).unwrap();
    let mut restored: World = serde_json::from_str(&serialized).unwrap();
    restored.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert_eq!(
        restored.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert_eq!(restored.active_in_hierarchy(child), Some(false));

    restored.reset_ecs_frame_performance_diagnostics();
    restored.set_parent_checked(child, None).unwrap();
    restored.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert_eq!(
        restored.world_transform(child).unwrap().translation,
        Vec3::new(4.0, 0.0, 0.0)
    );
    assert_eq!(restored.active_in_hierarchy(child), Some(true));
    assert_eq!(
        restored
            .nodes()
            .iter()
            .find(|node| node.id == child)
            .unwrap()
            .parent,
        None
    );

    let diagnostics = restored.ecs_frame_performance_diagnostics().derived_state;
    assert_eq!(diagnostics.hierarchy_parent_snapshot_entities, 0);
    assert_eq!(diagnostics.hierarchy_validity_entities, 0);
    assert_eq!(diagnostics.hierarchy_topology_rebuild_entities, 0);
    assert_eq!(diagnostics.active_propagation_entities, 1);
    assert_eq!(diagnostics.world_matrix_propagation_entities, 1);
    assert_eq!(diagnostics.node_cache_rebuilt_entities, 1);
}

#[test]
fn checked_reparent_and_removal_preserve_stable_subtree_order() {
    let mut world = World::empty();
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let first = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let second = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let unrelated_root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world.set_parent_checked(first, Some(root)).unwrap();
    world.set_parent_checked(second, Some(root)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert_eq!(
        world
            .subtree_records(root)
            .into_iter()
            .map(|record| record.id)
            .collect::<Vec<_>>(),
        vec![root, first, second]
    );

    world.set_parent_checked(first, None).unwrap();
    world.set_parent_checked(first, Some(root)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert_eq!(
        world
            .subtree_records(root)
            .into_iter()
            .map(|record| record.id)
            .collect::<Vec<_>>(),
        vec![root, first, second]
    );

    world.remove_entity(first).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert_eq!(
        world
            .subtree_records(root)
            .into_iter()
            .map(|record| record.id)
            .collect::<Vec<_>>(),
        vec![root, second]
    );
    assert_eq!(world.subtree_records(unrelated_root)[0].id, unrelated_root);
}

#[test]
fn active_hierarchy_propagates_inactive_and_reactivated_ancestors() {
    let mut world = World::new();
    let root = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let middle = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let leaf = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
        let entity = world
            .spawn_node(if index + 1 == LARGE_HIERARCHY_NODE_COUNT {
                NodeKind::Mesh
            } else {
                NodeKind::Cube
            })
            .expect("test scene spawn should succeed");
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
    let parent = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let mesh = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
