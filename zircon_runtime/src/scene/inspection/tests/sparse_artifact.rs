use super::*;

#[test]
fn five_thousand_node_name_edit_updates_one_parent_aggregate_without_sibling_scan() {
    const NODE_COUNT: usize = 5_000;

    let mut world = World::empty();
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let renamed = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world.set_parent_checked(renamed, Some(root)).unwrap();
    let mut unrelated = None;
    for index in 2..NODE_COUNT {
        let entity = world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        world.set_parent_checked(entity, Some(root)).unwrap();
        if index == NODE_COUNT / 2 {
            unrelated = Some(entity);
        }
    }
    let unrelated = unrelated.expect("large hierarchy should contain an unrelated sibling");
    let initial = world.inspection_artifact();
    let before = world.inspection_artifact_diagnostics();
    let unrelated_hash = initial
        .hierarchy_row(unrelated)
        .expect("unrelated sibling should have an inspection row")
        .subtree_hash;

    world.rename_node(renamed, "Renamed child").unwrap();
    let current = world.inspection_artifact();
    let after = world.inspection_artifact_diagnostics();
    let delta = current
        .published_delta_from(initial.generation())
        .expect("adjacent name edit should publish its bounded delta");
    let rebuilt_rows = world.inspection_artifact().hierarchy_rows().to_vec();

    assert_eq!(current.summary().node_count(), NODE_COUNT);
    assert_eq!(current.hierarchy_row_override_count(), 2);
    assert_eq!(current.hierarchy_child_hash_override_count(), 1);
    assert!(!current.hierarchy_rows_are_materialized());
    assert_eq!(after.hierarchy_builds(), before.hierarchy_builds() + 1);
    assert_eq!(
        after.hierarchy_rows_built() - before.hierarchy_rows_built(),
        2
    );
    assert_eq!(
        after.hierarchy_child_hash_updates() - before.hierarchy_child_hash_updates(),
        1
    );
    assert_eq!(
        delta
            .changed_rows()
            .iter()
            .map(|row| row.entity)
            .collect::<Vec<_>>(),
        vec![renamed, root]
    );
    assert_eq!(
        current
            .hierarchy_row(unrelated)
            .expect("unrelated sibling should remain addressable")
            .subtree_hash,
        unrelated_hash
    );
    for entity in [renamed, root] {
        assert_eq!(
            current.hierarchy_row(entity).map(|row| row.subtree_hash),
            rebuilt_rows
                .iter()
                .find(|row| row.entity == entity)
                .map(|row| row.subtree_hash)
        );
    }
}

#[test]
fn hundred_thousand_node_name_delta_stays_sparse_until_a_consumer_reads_the_complete_view() {
    const NODE_COUNT: usize = 100_000;

    let mut world = World::empty();
    let renamed = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    for _ in 1..NODE_COUNT {
        world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
    }
    let initial = world.inspection_artifact();
    let before = world.inspection_artifact_diagnostics();

    world
        .rename_node(renamed, "Measured large name delta")
        .unwrap();
    let current = world.inspection_artifact();
    let delta = current
        .published_delta_from(initial.generation())
        .expect("the adjacent generation should retain the bounded hierarchy delta");
    assert_eq!(delta.changed_rows().len(), 1);
    assert!(!delta.requires_hierarchy_reflow());
    assert!(!current.hierarchy_rows_are_materialized());
    let after_publish = world.inspection_artifact_diagnostics();
    assert_eq!(
        after_publish.hierarchy_builds(),
        before.hierarchy_builds() + 1
    );
    assert_eq!(
        after_publish.hierarchy_rows_built(),
        before.hierarchy_rows_built() + 1
    );
    assert_eq!(
        after_publish.hierarchy_child_hash_updates(),
        before.hierarchy_child_hash_updates()
    );
    assert_eq!(
        after_publish.hierarchy_full_materializations(),
        before.hierarchy_full_materializations()
    );
    assert_eq!(
        after_publish.hierarchy_rows_materialized(),
        before.hierarchy_rows_materialized()
    );

    let rows = current.hierarchy_rows_arc();
    assert_eq!(rows.len(), NODE_COUNT);
    let after = world.inspection_artifact_diagnostics();
    assert_eq!(
        after.hierarchy_full_materializations(),
        before.hierarchy_full_materializations() + 1
    );
    assert_eq!(
        after.hierarchy_rows_materialized(),
        before.hierarchy_rows_materialized() + NODE_COUNT as u64
    );
}

#[test]
fn cloned_world_materialization_diagnostics_do_not_mutate_the_source_cache() {
    let mut source = World::empty();
    let renamed = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    source.inspection_artifact();
    source.rename_node(renamed, "Sparse clone").unwrap();
    let source_artifact = source.inspection_artifact();
    assert!(!source_artifact.hierarchy_rows_are_materialized());

    let cloned = source.clone();
    let source_before = source.inspection_artifact_diagnostics();
    let clone_before = cloned.inspection_artifact_diagnostics();

    let cloned_rows = cloned.inspection_artifact().hierarchy_rows_arc();
    assert_eq!(cloned_rows.len(), 2);

    let source_after = source.inspection_artifact_diagnostics();
    let clone_after = cloned.inspection_artifact_diagnostics();
    assert_eq!(source_after, source_before);
    assert_eq!(
        clone_after.hierarchy_full_materializations(),
        clone_before.hierarchy_full_materializations() + 1
    );
    assert_eq!(
        clone_after.hierarchy_rows_materialized(),
        clone_before.hierarchy_rows_materialized() + 2
    );
}

#[test]
fn same_generation_sibling_renames_compose_parent_aggregate_updates() {
    let mut world = World::empty();
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let first = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let second = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world.set_parent_checked(first, Some(root)).unwrap();
    world.set_parent_checked(second, Some(root)).unwrap();
    let initial = world.inspection_artifact();
    let before = world.inspection_artifact_diagnostics();

    world.rename_node(first, "First renamed").unwrap();
    world.rename_node(second, "Second renamed").unwrap();
    let current = world.inspection_artifact();
    let after = world.inspection_artifact_diagnostics();
    let delta = current
        .published_delta_from(initial.generation())
        .expect("batched sibling names should publish one adjacent delta");

    let mut changed = delta
        .changed_rows()
        .iter()
        .map(|row| row.entity)
        .collect::<Vec<_>>();
    changed.sort_unstable();
    assert_eq!(changed, vec![root, first, second]);
    assert_eq!(current.hierarchy_row_override_count(), 3);
    assert_eq!(current.hierarchy_child_hash_override_count(), 1);
    assert!(!current.hierarchy_rows_are_materialized());
    assert_eq!(
        after.hierarchy_rows_built() - before.hierarchy_rows_built(),
        3
    );
    assert_eq!(
        after.hierarchy_child_hash_updates() - before.hierarchy_child_hash_updates(),
        2
    );
    let rebuilt_rows = world.inspection_artifact().hierarchy_rows().to_vec();
    for entity in [root, first, second] {
        assert_eq!(
            current.hierarchy_row(entity).map(|row| row.subtree_hash),
            rebuilt_rows
                .iter()
                .find(|row| row.entity == entity)
                .map(|row| row.subtree_hash)
        );
    }
}

#[test]
fn same_generation_ancestor_and_descendant_renames_propagate_deepest_first() {
    let mut world = World::empty();
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let parent = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world.set_parent_checked(parent, Some(root)).unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();
    let initial = world.inspection_artifact();
    let before = world.inspection_artifact_diagnostics();

    world.rename_node(parent, "Parent renamed").unwrap();
    world.rename_node(child, "Child renamed").unwrap();
    let current = world.inspection_artifact();
    let after = world.inspection_artifact_diagnostics();
    let delta = current
        .published_delta_from(initial.generation())
        .expect("batched ancestry names should publish one adjacent delta");

    assert_eq!(
        delta
            .changed_rows()
            .iter()
            .map(|row| row.entity)
            .collect::<Vec<_>>(),
        vec![child, parent, root]
    );
    assert_eq!(current.hierarchy_row_override_count(), 3);
    assert_eq!(current.hierarchy_child_hash_override_count(), 2);
    assert!(!current.hierarchy_rows_are_materialized());
    assert_eq!(
        after.hierarchy_rows_built() - before.hierarchy_rows_built(),
        3
    );
    assert_eq!(
        after.hierarchy_child_hash_updates() - before.hierarchy_child_hash_updates(),
        2
    );
    let rebuilt_rows = world.inspection_artifact().hierarchy_rows().to_vec();
    for entity in [root, parent, child] {
        assert_eq!(
            current.hierarchy_row(entity).map(|row| row.subtree_hash),
            rebuilt_rows
                .iter()
                .find(|row| row.entity == entity)
                .map(|row| row.subtree_hash)
        );
    }
}

#[test]
fn cyclic_hierarchy_name_change_falls_back_to_explicit_reflow() {
    let mut source = World::empty();
    let first = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let second = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let mut world =
        world_with_serialized_parents(&source, &[(first, Some(second)), (second, Some(first))]);
    let initial = world.inspection_artifact();
    let before = world.inspection_artifact_diagnostics();

    world.rename_node(first, "Cyclic rename").unwrap();
    let current = world.inspection_artifact();
    let after = world.inspection_artifact_diagnostics();
    let delta = current
        .published_delta_from(initial.generation())
        .expect("cycle fallback should still publish an adjacent reflow delta");

    assert!(delta.requires_hierarchy_reflow());
    assert_eq!(current.hierarchy_row_override_count(), 0);
    assert_eq!(current.hierarchy_child_hash_override_count(), 0);
    assert_eq!(
        after.hierarchy_rows_built() - before.hierarchy_rows_built(),
        2
    );
    assert_eq!(
        after.hierarchy_child_hash_updates(),
        before.hierarchy_child_hash_updates()
    );
    let rebuilt_rows = world.inspection_artifact().hierarchy_rows().to_vec();
    assert_eq!(current.hierarchy_rows(), rebuilt_rows.as_slice());
}

#[test]
fn sequential_name_edits_keep_the_hierarchy_sparse_without_periodic_full_materialization() {
    const NODE_COUNT: usize = 256;

    let mut world = World::empty();
    let entities = (0..NODE_COUNT)
        .map(|_| {
            world
                .spawn_node(NodeKind::Empty)
                .expect("test scene spawn should succeed")
        })
        .collect::<Vec<_>>();
    world.inspection_artifact();
    let before = world.inspection_artifact_diagnostics();

    for (index, entity) in entities.iter().copied().enumerate() {
        world
            .rename_node(entity, format!("Sparse {index}"))
            .unwrap();
        let current = world.inspection_artifact();
        assert!(!current.hierarchy_rows_are_materialized());
    }

    let current = world.inspection_artifact();
    let after = world.inspection_artifact_diagnostics();
    assert_eq!(current.hierarchy_row_override_count(), NODE_COUNT);
    assert_eq!(
        after.hierarchy_full_materializations(),
        before.hierarchy_full_materializations()
    );
    assert_eq!(
        after.hierarchy_rows_materialized(),
        before.hierarchy_rows_materialized()
    );
    assert_eq!(
        after.hierarchy_builds(),
        before.hierarchy_builds() + NODE_COUNT as u64
    );
    assert_eq!(
        after.hierarchy_rows_built(),
        before.hierarchy_rows_built() + NODE_COUNT as u64
    );
    for (index, entity) in entities.into_iter().enumerate() {
        assert_eq!(
            current
                .hierarchy_row(entity)
                .expect("renamed entity should remain directly addressable")
                .display_name,
            format!("Sparse {index}")
        );
    }
}

#[test]
fn hundred_thousand_node_transform_change_publishes_only_the_target_field_delta() {
    const NODE_COUNT: usize = 100_000;

    let mut world = World::empty();
    let mut target = 0;
    for index in 0..NODE_COUNT {
        let entity = world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        if index == NODE_COUNT / 2 {
            target = entity;
        }
    }
    let initial = world.inspection_artifact();
    let initial_fields = world
        .inspection_fields_artifact(target)
        .expect("target should expose reflected fields");
    assert_eq!(initial.hierarchy_rows().len(), NODE_COUNT);

    world
        .update_transform(
            target,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .unwrap();
    let current = world.inspection_artifact();
    let delta = current
        .published_delta_from(initial.generation())
        .expect("the adjacent generation should retain the bounded notification delta");

    assert!(delta.added_rows().is_empty());
    assert!(delta.changed_rows().is_empty());
    assert!(delta.removed_entities().is_empty());
    let current_fields = world
        .inspection_fields_artifact(target)
        .expect("target should retain reflected fields");
    let fields_delta = current_fields.delta_from(&initial_fields);
    assert_eq!(fields_delta.entity(), target);
    assert!(!fields_delta.changed_fields().is_empty());
    assert!(fields_delta.removed_fields().is_empty());
    let diagnostics = world.inspection_artifact_diagnostics();
    assert_eq!(diagnostics.hierarchy_builds(), 1);
    assert_eq!(diagnostics.hierarchy_rows_built(), NODE_COUNT as u64);
    assert_eq!(diagnostics.focused_field_builds(), 2);
}
