use crate::scene::components::Name;
use crate::scene::ecs::Mut;
use crate::scene::{EntityId, NodeKind, SceneError, World};

fn world_facts(world: &World) -> (serde_json::Value, u64, u64, usize) {
    (
        serde_json::to_value(world).unwrap(),
        world.world_generation(),
        world.lifecycle_visibility_revision(),
        world.staged_lifecycle_events.len(),
    )
}

#[test]
fn preparation_normalizes_roots_and_counts_only_affected_cameras() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let camera = world.spawn_node(NodeKind::Camera).unwrap();
    let child = world.spawn_node(NodeKind::Empty).unwrap();
    let survivor = world.spawn_node(NodeKind::Camera).unwrap();
    world.set_parent_checked(camera, Some(root)).unwrap();
    world.set_parent_checked(child, Some(camera)).unwrap();
    world.set_active_camera(camera);
    let before = world_facts(&world);

    let prepared = world
        .prepare_entity_subtrees([child, camera, root, root])
        .unwrap();
    assert_eq!(prepared.normalized_roots(), [root]);
    assert_eq!(prepared.affected_entity_count(), 3);
    assert_eq!(prepared.affected_camera_count(), 1);
    assert_eq!(prepared.world_camera_count(), 2);
    assert_eq!(prepared.world_generation(), before.1);
    assert_eq!(world_facts(&world), before);

    let batch = world.remove_prepared_entity_subtrees(prepared).unwrap();
    assert_eq!(
        batch.entity_ids().collect::<Vec<_>>(),
        [root, camera, child]
    );
    assert_eq!(world.camera_count(), 1);
    assert_eq!(world.active_camera(), survivor);
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(world.camera_count(), 2);
    assert_eq!(world.active_camera(), camera);
    assert_eq!(world.parent_of(child), Some(camera));
}

#[test]
fn stale_preparation_rejects_camera_reparenting_without_detaching_rows() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let camera = world.spawn_node(NodeKind::Camera).unwrap();
    let prepared = world.prepare_entity_subtrees([root]).unwrap();
    assert_eq!(prepared.affected_camera_count(), 0);
    let prepared_generation = prepared.world_generation();
    world.set_parent_checked(camera, Some(root)).unwrap();
    let before = world_facts(&world);

    assert!(matches!(
        world.remove_prepared_entity_subtrees(prepared),
        Err(SceneError::DetachedEntityPreparationStale {
            prepared_generation: captured,
            current_generation,
        }) if captured == prepared_generation && current_generation == before.1
    ));
    assert_eq!(world_facts(&world), before);
    let diagnostics = world
        .ecs_frame_performance_diagnostics()
        .detached_entity_batches;
    assert_eq!(diagnostics.commit_count, 0);
    assert_eq!(diagnostics.lifecycle_events, 0);
    assert_eq!(diagnostics.rejected_preflights, 1);

    let refreshed = world.prepare_entity_subtrees([root]).unwrap();
    assert_eq!(refreshed.affected_camera_count(), 1);
    assert_eq!(refreshed.world_camera_count(), 1);
    drop(refreshed);
    assert_eq!(world_facts(&world), before);
}

#[test]
fn preparation_rejects_unflushed_component_mutations() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let prepared = world.prepare_entity_subtrees([root]).unwrap();
    let mut query = world.query::<Mut<'static, Name>>();
    query.get_mut(&mut world, root).unwrap().0 = "Changed after prepare".to_owned();
    let before = world_facts(&world);

    assert!(matches!(
        world.remove_prepared_entity_subtrees(prepared),
        Err(SceneError::DetachedEntityPreparationStale { .. })
    ));
    assert_eq!(world_facts(&world), before);
}

#[test]
fn preparation_cannot_cross_world_clone_or_persistence_boundaries() {
    let mut source = World::empty();
    let root = source.spawn_node(NodeKind::Empty).unwrap();
    let decoded = serde_json::from_value(serde_json::to_value(&source).unwrap()).unwrap();
    let mut independent = World::empty();
    assert_eq!(independent.spawn_node(NodeKind::Empty).unwrap(), root);

    for mut target in [source.clone(), decoded, independent] {
        // Equal ids and revisions must not grant access to another World's rows.
        target.world_generation = source.world_generation;
        assert_eq!(target.world_generation(), source.world_generation());
        let prepared = source.prepare_entity_subtrees([root]).unwrap();
        let before = world_facts(&target);
        assert!(matches!(
            target.remove_prepared_entity_subtrees(prepared),
            Err(SceneError::DetachedEntityPreparationWorldMismatch)
        ));
        assert_eq!(world_facts(&target), before);
        assert!(source.contains_entity(root));
    }
}

#[test]
fn preparation_survives_world_move_but_not_an_earlier_detach() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let first = world.prepare_entity_subtrees([root]).unwrap();
    let second = world.prepare_entity_subtrees([root]).unwrap();
    let mut moved = Box::new(world);

    let batch = moved.remove_prepared_entity_subtrees(first).unwrap();
    let before = world_facts(&moved);
    assert!(matches!(
        moved.remove_prepared_entity_subtrees(second),
        Err(SceneError::DetachedEntityPreparationStale { .. })
    ));
    assert_eq!(world_facts(&moved), before);
    moved.restore_detached_entity_batch(batch).unwrap();
    assert!(moved.contains_entity(root));
}

#[test]
fn preparation_includes_unflushed_hierarchy_changes_in_camera_counts() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let camera = world.spawn_node(NodeKind::Camera).unwrap();
    assert!(world.corrupt_hierarchy_parent_with_pending_mutation_for_tests(camera, Some(root)));
    let before = world_facts(&world);

    let prepared = world.prepare_entity_subtrees([root]).unwrap();
    assert_eq!(prepared.affected_entity_count(), 2);
    assert_eq!(prepared.affected_camera_count(), 1);
    assert_eq!(prepared.world_camera_count(), 1);
    assert_eq!(world_facts(&world), before);
}

#[test]
fn exhausted_world_generation_cannot_issue_a_preparation() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    world.world_generation = world.world_generation.advanced_by(u64::MAX);
    let before = world_facts(&world);

    assert!(matches!(
        world.prepare_entity_subtrees([root]),
        Err(SceneError::DetachedEntityPreparationGenerationExhausted)
    ));
    assert_eq!(world_facts(&world), before);
}

#[test]
#[ignore = "managed Runtime08 2/128 camera subtree with 100k unrelated entity gate"]
fn prepared_camera_subtree_managed_scale_fixture() {
    const UNRELATED_ENTITIES: EntityId = 100_000;
    for camera_count in [2, 128] {
        let mut world = World::empty();
        let root = world.spawn_node(NodeKind::Empty).unwrap();
        for _ in 0..camera_count {
            let camera = world.spawn_node(NodeKind::Camera).unwrap();
            world.set_parent_checked(camera, Some(root)).unwrap();
        }
        for offset in 0..UNRELATED_ENTITIES {
            world.spawn_empty_at(1_000_000 + offset).unwrap();
        }
        world.reset_ecs_frame_performance_diagnostics();
        let before = world_facts(&world);
        let prepared = world.prepare_entity_subtrees([root]).unwrap();
        assert_eq!(prepared.normalized_roots(), [root]);
        assert_eq!(prepared.affected_entity_count(), camera_count + 1);
        assert_eq!(prepared.affected_camera_count(), camera_count);
        assert_eq!(prepared.world_camera_count(), camera_count);
        drop(prepared);
        assert_eq!(world_facts(&world), before);
        let diagnostics = world
            .ecs_frame_performance_diagnostics()
            .detached_entity_batches;
        assert_eq!(diagnostics.commit_count, 0);
        assert_eq!(diagnostics.lifecycle_events, 0);
        assert_eq!(diagnostics.full_world_clone_bytes, 0);
        assert_eq!(diagnostics.node_record_clone_bytes, 0);
        assert_eq!(diagnostics.rollback_bytes, 0);
    }
}
