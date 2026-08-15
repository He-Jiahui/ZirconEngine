use super::*;

#[test]
fn query_state_cached_iteration_rebuilds_only_for_new_matching_archetypes() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let enemy = world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();

    let mut query = world.query_filtered::<(EntityId, &Health), Without<Enemy>>();
    assert_eq!(query.cache_rebuilds(), 1);
    assert_eq!(query.cached_entity_count(), 1);
    assert_eq!(query.cached_archetype_count(), 1);
    assert_eq!(query.cached_archetype_plans()[0].bindings().len(), 1);
    assert!(format!("{:?}", query.cached_archetype_plans()[0].bindings()).contains("Table"));
    let initial_revision = query.cached_revision();

    let first = query
        .iter_cached(&world)
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(first, vec![(player, 10)]);
    assert_eq!(query.cache_rebuilds(), 1);

    world.insert(player, Health(11)).unwrap();
    let replaced = query
        .iter_cached(&world)
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(replaced, vec![(player, 11)]);
    assert_eq!(query.cache_rebuilds(), 1);
    assert_eq!(query.cached_revision(), initial_revision);

    let prop = world.spawn((Name("Prop".to_string()), Health(2))).unwrap();
    let after_spawn = query
        .iter_cached(&world)
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(after_spawn, vec![(player, 11), (prop, 2)]);
    assert_eq!(query.cache_rebuilds(), 2);
    assert_eq!(query.cached_entity_count(), 2);
    assert!(query.cached_revision() > initial_revision);

    world.remove::<Health>(player).unwrap();
    let after_remove = query
        .iter_cached(&world)
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(after_remove, vec![(prop, 2)]);
    assert_eq!(query.cache_rebuilds(), 2);
    assert_eq!(query.cached_entity_count(), 1);
    assert_eq!(world.get::<Health>(enemy), Some(&Health(4)));
}

#[test]
fn query_state_count_and_empty_helpers_can_use_cached_candidates() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let enemy = world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();
    let prop = world.spawn((Name("Prop".to_string()),)).unwrap();

    let mut query = world.query_filtered::<(EntityId, &Health), With<Player>>();
    assert_eq!(query.count(&world), 1);
    assert!(!query.is_empty(&world));
    assert!(query.contains(&world, player));
    assert!(!query.contains(&world, enemy));
    assert!(!query.contains(&world, prop));
    assert!(!query.contains(&world, 999));
    assert_eq!(
        query
            .get(&world, player)
            .map(|(entity, health)| (entity, health.0)),
        Ok((player, 10))
    );
    assert_eq!(
        query
            .get_many(&world, [player, player])
            .map(|items| items.map(|(entity, health)| (entity, health.0))),
        Ok([(player, 10), (player, 10)])
    );
    let unique_player = UniqueEntityArray::new([player]).unwrap();
    assert_eq!(
        query
            .get_many_unique(&world, unique_player)
            .map(|items| items.map(|(entity, health)| (entity, health.0))),
        Ok([(player, 10)])
    );
    assert_eq!(
        UniqueEntityArray::new([player, player]),
        Err(QueryEntityError::DuplicateEntity(player))
    );
    assert_eq!(
        query
            .iter_many(&world, [enemy, player, 999, player])
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>(),
        vec![(player, 10), (player, 10)]
    );
    assert_eq!(
        query
            .iter_many_cached(&world, [enemy, player, 999, player])
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>(),
        vec![(player, 10), (player, 10)]
    );
    let borrowed_entities = vec![enemy, player, player];
    assert_eq!(
        query
            .iter_many(&world, &borrowed_entities)
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>(),
        vec![(player, 10), (player, 10)]
    );
    assert_eq!(
        query
            .iter_many_cached(&world, &borrowed_entities)
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>(),
        vec![(player, 10), (player, 10)]
    );
    assert_eq!(
        query
            .get_many(&world, [player, enemy])
            .map(|items| items.map(|(entity, health)| (entity, health.0))),
        Err(QueryEntityError::QueryDoesNotMatch(enemy))
    );
    assert_eq!(
        query
            .get_many_unique(&world, UniqueEntityArray::new([player, enemy]).unwrap())
            .map(|items| items.map(|(entity, health)| (entity, health.0))),
        Err(QueryEntityError::QueryDoesNotMatch(enemy))
    );
    assert_eq!(
        query.get(&world, enemy),
        Err(QueryEntityError::QueryDoesNotMatch(enemy))
    );
    assert_eq!(
        query.get(&world, 999),
        Err(QueryEntityError::NotSpawned(999))
    );
    assert_eq!(query.cache_rebuilds(), 1);
    assert_eq!(query.count_cached(&world), 1);
    assert!(!query.is_empty_cached(&world));
    assert!(query.contains_cached(&world, player));
    assert!(!query.contains_cached(&world, enemy));
    assert_eq!(
        query
            .get_cached(&world, player)
            .map(|(entity, health)| (entity, health.0)),
        Ok((player, 10))
    );
    assert_eq!(
        query
            .get_many_cached(&world, [player, player])
            .map(|items| items.map(|(entity, health)| (entity, health.0))),
        Ok([(player, 10), (player, 10)])
    );
    assert_eq!(
        query
            .get_many_unique_cached(&world, unique_player)
            .map(|items| items.map(|(entity, health)| (entity, health.0))),
        Ok([(player, 10)])
    );
    assert_eq!(
        UniqueEntityArray::new([player, player]),
        Err(QueryEntityError::DuplicateEntity(player))
    );
    assert_eq!(
        query
            .iter_many_unique(&world, unique_player)
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>(),
        vec![(player, 10)]
    );
    assert_eq!(
        query
            .iter_many_unique_cached(&world, unique_player)
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>(),
        vec![(player, 10)]
    );
    assert_eq!(
        query
            .get_many_cached(&world, [player, enemy])
            .map(|items| items.map(|(entity, health)| (entity, health.0))),
        Err(QueryEntityError::QueryDoesNotMatch(enemy))
    );
    assert_eq!(
        query.get_cached(&world, enemy),
        Err(QueryEntityError::QueryDoesNotMatch(enemy))
    );
    assert_eq!(
        query
            .single_cached(&world)
            .map(|(entity, health)| (entity, health.0)),
        Ok((player, 10))
    );
    assert_eq!(query.cache_rebuilds(), 1);

    world.remove::<Player>(player).unwrap();
    assert_eq!(query.count_cached(&world), 0);
    assert!(query.is_empty_cached(&world));
    assert!(!query.contains_cached(&world, player));
    assert!(query
        .iter_many_cached(&world, [player, enemy])
        .collect::<Vec<_>>()
        .is_empty());
    assert_eq!(query.cache_rebuilds(), 1);

    world.insert(enemy, Player).unwrap();
    assert_eq!(
        query
            .iter_many_cached(&world, [player, enemy, enemy])
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>(),
        vec![(enemy, 4), (enemy, 4)]
    );
    assert_eq!(query.count_cached_direct(&world), 1);
    assert!(!query.is_empty_cached_direct(&world));
    assert!(query.contains_cached_direct(&world, enemy));
    assert!(!query.contains_cached_direct(&world, player));
    assert_eq!(
        query
            .get_cached_direct(&world, enemy)
            .map(|(entity, health)| (entity, health.0)),
        Ok((enemy, 4))
    );
    assert_eq!(
        query
            .get_many_cached_direct(&world, [enemy, enemy])
            .map(|items| items.map(|(entity, health)| (entity, health.0))),
        Ok([(enemy, 4), (enemy, 4)])
    );
    assert_eq!(
        query
            .get_many_unique_cached_direct(&world, UniqueEntityArray::new([enemy]).unwrap())
            .map(|items| items.map(|(entity, health)| (entity, health.0))),
        Ok([(enemy, 4)])
    );
    assert_eq!(
        UniqueEntityArray::new([enemy, enemy]),
        Err(QueryEntityError::DuplicateEntity(enemy))
    );
    assert_eq!(
        query
            .iter_many_cached_direct(&world, [player, enemy, enemy, 999])
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>(),
        vec![(enemy, 4), (enemy, 4)]
    );
    assert_eq!(
        query
            .iter_many_unique_cached_direct(&world, UniqueEntityArray::new([enemy]).unwrap())
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>(),
        vec![(enemy, 4)]
    );
    assert_eq!(
        query
            .get_many_cached_direct(&world, [enemy, player])
            .map(|items| items.map(|(entity, health)| (entity, health.0))),
        Err(QueryEntityError::QueryDoesNotMatch(player))
    );
    assert_eq!(
        query.get_cached_direct(&world, player),
        Err(QueryEntityError::QueryDoesNotMatch(player))
    );
    assert_eq!(
        query
            .single_cached_direct(&world)
            .map(|(entity, health)| (entity, health.0)),
        Ok((enemy, 4))
    );
    assert_eq!(query.cache_rebuilds(), 2);

    let mut optional_query = world.query::<(EntityId, Option<&Health>)>();
    assert!(optional_query.contains(&world, prop));
    assert!(optional_query.contains_cached(&world, prop));
    assert!(!optional_query.contains(&world, 999));
    assert!(!optional_query.contains_cached(&world, 999));
    assert_eq!(
        optional_query
            .get(&world, prop)
            .map(|(entity, health)| (entity, health.map(|health| health.0))),
        Ok((prop, None))
    );
    assert_eq!(
        optional_query
            .get_many(&world, [prop, enemy])
            .map(|items| items.map(|(entity, health)| (entity, health.map(|health| health.0)))),
        Ok([(prop, None), (enemy, Some(4))])
    );
    assert_eq!(
        optional_query
            .iter_many(&world, [prop, 999, enemy])
            .map(|(entity, health)| (entity, health.map(|health| health.0)))
            .collect::<Vec<_>>(),
        vec![(prop, None), (enemy, Some(4))]
    );
    assert_eq!(
        optional_query
            .iter_many_cached(&world, [prop, 999, enemy])
            .map(|(entity, health)| (entity, health.map(|health| health.0)))
            .collect::<Vec<_>>(),
        vec![(prop, None), (enemy, Some(4))]
    );
    assert_eq!(
        optional_query
            .iter_many_cached_direct(&world, [prop, 999, enemy])
            .map(|(entity, health)| (entity, health.map(|health| health.0)))
            .collect::<Vec<_>>(),
        vec![(prop, None), (enemy, Some(4))]
    );
    assert_eq!(
        optional_query
            .get_cached(&world, prop)
            .map(|(entity, health)| (entity, health.map(|health| health.0))),
        Ok((prop, None))
    );
    assert_eq!(
        optional_query
            .get_many_cached(&world, [prop, enemy])
            .map(|items| items.map(|(entity, health)| (entity, health.map(|health| health.0)))),
        Ok([(prop, None), (enemy, Some(4))])
    );
}

#[test]
fn query_state_cached_direct_iteration_reads_compiled_archetype_slots() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let enemy = world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();

    let mut query = world.query::<(EntityId, &Health)>();
    let first = query
        .iter_cached_direct(&world)
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();

    assert_eq!(first, vec![(player, 10), (enemy, 4)]);
    assert_eq!(query.cached_entity_count(), 2);
    assert_eq!(query.cached_archetype_count(), 1);
    let bindings = query.cached_archetype_plans()[0].bindings();
    assert_eq!(bindings.len(), 1);
    assert_eq!(
        bindings[0].component_id(),
        world.registered_component_id::<Health>().unwrap()
    );
    assert!(format!("{:?}", bindings[0]).contains("column_slot"));
    assert_eq!(query.cache_rebuilds(), 1);

    world.remove::<Health>(player).unwrap();
    let after_remove = query
        .iter_cached_direct(&world)
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();

    assert_eq!(after_remove, vec![(enemy, 4)]);
    assert_eq!(query.cache_rebuilds(), 1);
    assert_eq!(query.cached_entity_count(), 1);
}

#[test]
fn query_state_cached_direct_iteration_preserves_optional_and_ref_items() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let prop = world.spawn((Name("Prop".to_string()),)).unwrap();

    let mut optional_query = world.query::<(EntityId, Option<&Health>)>();
    let optional_rows = optional_query
        .iter_cached_direct(&world)
        .map(|(entity, health)| (entity, health.map(|health| health.0)))
        .collect::<Vec<_>>();

    assert_eq!(optional_rows, vec![(player, Some(10)), (prop, None)]);

    let mut ref_query = world.query::<(EntityId, Ref<'static, Health>)>();
    let ref_rows = ref_query
        .iter_cached_direct(&world)
        .map(|(entity, health)| (entity, health.0, health.is_added(), health.is_changed()))
        .collect::<Vec<_>>();

    assert_eq!(ref_rows, vec![(player, 10, true, true)]);
}

#[test]
fn query_state_cached_direct_iteration_reads_sparse_plan_bindings() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), SparseScore(3), Player))
        .unwrap();
    let enemy = world
        .spawn((Name("Enemy".to_string()), SparseScore(9), Enemy))
        .unwrap();
    world.spawn((Name("Prop".to_string()),)).unwrap();

    let mut query = world.query::<(EntityId, &SparseScore)>();
    let score_id = world.registered_component_id::<SparseScore>().unwrap();
    let rows = query
        .iter_cached_direct(&world)
        .map(|(entity, score)| (entity, score.0))
        .collect::<Vec<_>>();

    assert_eq!(rows, vec![(player, 3), (enemy, 9)]);
    assert_eq!(query.cached_entity_count(), 2);
    assert!(query.cached_archetype_plans().iter().all(|plan| {
        plan.bindings().len() == 1
            && plan.bindings()[0].component_id() == score_id
            && format!("{:?}", plan.bindings()[0]).contains("SparseSet")
    }));

    world.remove::<SparseScore>(player).unwrap();
    let after_remove = query
        .iter_cached_direct(&world)
        .map(|(entity, score)| (entity, score.0))
        .collect::<Vec<_>>();

    assert_eq!(after_remove, vec![(enemy, 9)]);
    assert_eq!(query.cached_entity_count(), 1);
}

#[test]
fn cached_sparse_optional_rows_and_tick_filters_use_compiled_locations() {
    let mut world = World::empty();
    let present = world
        .spawn((Name("Present sparse score".to_string()), SparseScore(3)))
        .unwrap();
    let absent = world
        .spawn((Name("Absent sparse score".to_string()),))
        .unwrap();

    let mut optional = world.query::<(EntityId, Option<&SparseScore>)>();
    let initial = optional
        .iter_cached(&world)
        .map(|(entity, score)| (entity, score.map(|score| score.0)))
        .collect::<Vec<_>>();
    assert_eq!(initial, vec![(present, Some(3)), (absent, None)]);

    world.clear_trackers();
    world.insert(absent, SparseScore(8)).unwrap();
    let mut added = world.query_filtered::<(EntityId, &SparseScore), Added<SparseScore>>();
    let added_rows = added
        .iter_cached(&world)
        .map(|(entity, score)| (entity, score.0))
        .collect::<Vec<_>>();
    assert_eq!(added_rows, vec![(absent, 8)]);

    world.clear_trackers();
    world.insert(present, SparseScore(4)).unwrap();
    let mut changed = world.query_filtered::<(EntityId, &SparseScore), Changed<SparseScore>>();
    let changed_rows = changed
        .iter_cached(&world)
        .map(|(entity, score)| (entity, score.0))
        .collect::<Vec<_>>();
    assert_eq!(changed_rows, vec![(present, 4)]);
}

#[test]
fn entity_locations_track_archetype_signature_changes() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Tracked".to_string()), Health(10)))
        .unwrap();

    let initial_archetype = world
        .internal_entity_location(entity)
        .unwrap()
        .location
        .archetype_id;
    assert_ne!(initial_archetype, ArchetypeId::EMPTY);

    world.insert(entity, Player).unwrap();
    let with_player_archetype = world
        .internal_entity_location(entity)
        .unwrap()
        .location
        .archetype_id;
    assert_ne!(with_player_archetype, initial_archetype);

    world.remove::<Player>(entity).unwrap();
    let restored_archetype = world
        .internal_entity_location(entity)
        .unwrap()
        .location
        .archetype_id;
    assert_eq!(restored_archetype, initial_archetype);
}

#[test]
fn query_state_cached_archetypes_do_not_require_optional_reads() {
    let mut world = World::empty();
    let with_health = world
        .spawn((Name("With Health".to_string()), Health(7)))
        .unwrap();
    let without_health = world
        .spawn((Name("Without Health".to_string()), Player))
        .unwrap();

    let mut query = world.query::<(EntityId, Option<&Health>)>();
    let health_id = world.registered_component_id::<Health>().unwrap();
    assert!(query.access().reads().contains(&health_id));
    assert!(!query.access().with().contains(&health_id));

    let rows = query
        .iter_cached(&world)
        .map(|(entity, health)| (entity, health.map(|health| health.0)))
        .collect::<Vec<_>>();

    assert_eq!(rows, vec![(with_health, Some(7)), (without_health, None)]);
    assert_eq!(
        query.cached_archetype_generation(),
        world.archetype_generation()
    );
    assert!(query.cached_archetype_count() >= 2);
}
