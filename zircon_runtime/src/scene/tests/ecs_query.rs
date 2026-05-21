use crate::scene::components::{Name, RenderLayerMask};
use crate::scene::ecs::{
    ArchetypeId, Changed, Component, ComponentStorageLocation, Mut, QueryEntityError,
    QuerySingleError, QueryState, Ref, StableEntityLocation, StorageType, With, Without,
};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Enemy;

impl Component for Enemy {}

#[derive(Debug, PartialEq, Eq)]
struct Player;

impl Component for Player {}

#[derive(Debug, PartialEq, Eq)]
struct SparseScore(u32);

impl Component for SparseScore {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

fn expect_query_error<T>(result: Result<T, QueryEntityError>) -> QueryEntityError {
    match result {
        Ok(_) => panic!("expected query error"),
        Err(error) => error,
    }
}

#[test]
fn query_state_reads_required_optional_and_entity_items_with_filters() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let enemy = world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();
    let prop = world.spawn((Name("Prop".to_string()),)).unwrap();

    let health_query =
        world.query_filtered::<(EntityId, &Health, Option<&Player>), Without<Enemy>>();
    let rows = health_query.iter(&world).collect::<Vec<_>>();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, player);
    assert_eq!(rows[0].1, &Health(10));
    assert!(rows[0].2.is_some());

    let named_query = world.query_filtered::<(EntityId, &Name, Option<&Health>), Without<Player>>();
    let named_rows = named_query
        .iter(&world)
        .map(|(entity, name, health)| (entity, name.0.as_str(), health.map(|health| health.0)))
        .collect::<Vec<_>>();

    assert_eq!(
        named_rows,
        vec![(enemy, "Enemy", Some(4)), (prop, "Prop", None)]
    );
}

#[test]
fn query_state_supports_five_item_data_and_filter_tuples() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();
    world.spawn((Name("Prop".to_string()),)).unwrap();

    type Data<'query> = (
        EntityId,
        &'query Name,
        &'query Health,
        Option<&'query Player>,
        Option<&'query Enemy>,
    );
    type Filters = (With<Player>, Without<Enemy>, (), (), ());

    let query = world.query_filtered::<Data<'static>, Filters>();
    let rows = query
        .iter(&world)
        .map(|(entity, name, health, player, enemy)| {
            (
                entity,
                name.0.clone(),
                health.0,
                player.is_some(),
                enemy.is_some(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(rows, vec![(player, "Player".to_string(), 10, true, false)]);

    let mut cached_query = world.query_filtered::<Data<'static>, Filters>();
    let cached_rows = cached_query
        .iter_cached_direct(&world)
        .map(|(entity, name, health, player, enemy)| {
            (
                entity,
                name.0.clone(),
                health.0,
                player.is_some(),
                enemy.is_some(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(cached_rows, rows);
}

#[test]
fn query_state_reads_stable_entity_location_as_query_data() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let enemy = world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();

    let expected_player_location = world.internal_entity_location(player).unwrap();
    let expected_enemy_location = world.internal_entity_location(enemy).unwrap();

    let mut query = world.query::<(EntityId, StableEntityLocation, &Health)>();
    let rows = query
        .iter(&world)
        .map(|(entity, location, health)| (entity, location, health.0))
        .collect::<Vec<_>>();

    assert_eq!(
        rows,
        vec![
            (player, expected_player_location, 10),
            (enemy, expected_enemy_location, 4)
        ]
    );

    let cached_rows = query
        .iter_cached_direct(&world)
        .map(|(entity, location, health)| (entity, location, health.0))
        .collect::<Vec<_>>();

    assert_eq!(cached_rows, rows);
}

#[test]
fn query_state_single_reports_zero_one_many_matches() {
    let mut world = World::empty();

    let empty_query = world.query::<&Health>();
    assert_eq!(
        empty_query.single(&world).unwrap_err(),
        QuerySingleError::NoEntities
    );

    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let one_query = world.query::<&Health>();
    assert_eq!(one_query.single(&world), Ok(&Health(10)));

    let mut cached_player_query = world.query_filtered::<(EntityId, &Health), With<Player>>();
    let (cached_entity, cached_health) = cached_player_query.single_cached(&world).unwrap();
    assert_eq!(cached_entity, player);
    assert_eq!(cached_health, &Health(10));

    let (direct_entity, direct_health) = cached_player_query.single_cached_direct(&world).unwrap();
    assert_eq!(direct_entity, player);
    assert_eq!(direct_health, &Health(10));

    world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();
    let mut many_query = world.query::<&Health>();
    assert_eq!(
        many_query.single(&world).unwrap_err(),
        QuerySingleError::MultipleEntities
    );
    assert_eq!(
        many_query.single_cached(&world).unwrap_err(),
        QuerySingleError::MultipleEntities
    );
}

#[test]
fn query_state_mutates_matching_components_without_touching_filtered_entities() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let enemy = world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();

    let mut query = world.query_filtered::<&mut Health, With<Player>>();
    let health_id = world.registered_component_id::<Health>().unwrap();
    assert_eq!(query.cached_location_count(), 1);
    assert_eq!(query.cached_locations()[0].stable_id, player);
    assert_eq!(
        query.cached_component_locations()[0][0].component_id,
        health_id
    );
    assert_eq!(query.cache_rebuilds(), 1);
    query.for_each_mut(&mut world, |health| health.0 += 5);

    assert_eq!(world.get::<Health>(player), Some(&Health(15)));
    assert_eq!(world.get::<Health>(enemy), Some(&Health(4)));
    assert_eq!(query.cache_rebuilds(), 1);

    let ally = world
        .spawn((Name("Ally".to_string()), Health(7), Player))
        .unwrap();
    query.for_each_mut(&mut world, |health| health.0 += 2);

    assert_eq!(world.get::<Health>(player), Some(&Health(17)));
    assert_eq!(world.get::<Health>(ally), Some(&Health(9)));
    assert_eq!(world.get::<Health>(enemy), Some(&Health(4)));
    assert_eq!(query.cache_rebuilds(), 2);
}

#[test]
fn query_state_get_mut_helpers_mutate_targets_and_reject_aliases() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let ally = world
        .spawn((Name("Ally".to_string()), Health(7), Player))
        .unwrap();
    let enemy = world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();

    let mut query = world.query_filtered::<&mut Health, With<Player>>();
    {
        let health = query.get_mut(&mut world, player).unwrap();
        health.0 += 5;
    }
    assert_eq!(world.get::<Health>(player), Some(&Health(15)));
    let enemy_error = expect_query_error(query.get_mut(&mut world, enemy));
    assert_eq!(enemy_error, QueryEntityError::QueryDoesNotMatch(enemy));
    let missing_error = expect_query_error(query.get_mut(&mut world, 999));
    assert_eq!(missing_error, QueryEntityError::NotSpawned(999));

    {
        let healths = query.get_many_mut(&mut world, [ally, player]).unwrap();
        healths[0].0 += 2;
        healths[1].0 += 3;
    }
    assert_eq!(world.get::<Health>(ally), Some(&Health(9)));
    assert_eq!(world.get::<Health>(player), Some(&Health(18)));
    assert_eq!(world.get::<Health>(enemy), Some(&Health(4)));
    let many_enemy_error = expect_query_error(query.get_many_mut(&mut world, [player, enemy]));
    assert_eq!(many_enemy_error, QueryEntityError::QueryDoesNotMatch(enemy));
    let alias_error = expect_query_error(query.get_many_mut(&mut world, [player, player]));
    assert_eq!(alias_error, QueryEntityError::AliasedMutability(player));

    {
        let requested = vec![enemy, ally, 999, ally, player];
        let mut iter = query.iter_many_mut(&mut world, &requested);
        while let Some(health) = iter.fetch_next() {
            health.0 += 1;
        }
    }
    assert_eq!(world.get::<Health>(ally), Some(&Health(11)));
    assert_eq!(world.get::<Health>(player), Some(&Health(19)));
    assert_eq!(world.get::<Health>(enemy), Some(&Health(4)));
}

#[test]
fn query_access_detects_conflicts_and_filter_disjointness() {
    let mut world = World::empty();
    world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();

    let read_health = world.query::<&Health>();
    let write_health = world.query::<&mut Health>();
    let write_players = world.query_filtered::<&mut Health, With<Player>>();
    let write_non_players = world.query_filtered::<&mut Health, Without<Player>>();

    assert!(read_health.conflicts_with(&write_health));
    assert!(write_health.conflicts_with(&write_players));
    assert!(!write_players.conflicts_with(&write_non_players));
}

#[test]
fn query_access_rejects_duplicate_mutable_component_in_one_query() {
    let mut world = World::empty();
    world
        .spawn((Name("Entity".to_string()), Health(1)))
        .unwrap();

    let error = QueryState::<(&mut Health, &mut Health)>::try_new(&mut world).unwrap_err();

    assert!(error.to_string().contains("mutably"));
}

#[test]
fn fixed_scene_components_are_queryable_through_m3_api() {
    let mut world = World::empty();
    let entity = world
        .spawn((
            Name("Renderable".to_string()),
            RenderLayerMask(0b0101),
            Health(8),
        ))
        .unwrap();

    let query = world.query_filtered::<(EntityId, &Name, &RenderLayerMask), With<Health>>();
    let rows = query.iter(&world).collect::<Vec<_>>();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, entity);
    assert_eq!(rows[0].1 .0, "Renderable");
    assert_eq!(rows[0].2 .0, 0b0101);
}

#[test]
fn ref_and_mut_query_items_report_change_ticks() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Tracked".to_string()), Health(10)))
        .unwrap();

    type ReadTracked = QueryState<(EntityId, Ref<'static, Health>)>;
    let mut read_system = crate::scene::ecs::SystemState::<ReadTracked>::new(&mut world).unwrap();
    let first = read_system.run(&mut world, |query| {
        query
            .iter()
            .map(|(entity, health)| (entity, health.0, health.is_added(), health.is_changed()))
            .collect::<Vec<_>>()
    });
    assert_eq!(first, vec![(entity, 10, true, true)]);

    let second = read_system.run(&mut world, |query| {
        query
            .iter()
            .map(|(entity, health)| (entity, health.is_added(), health.is_changed()))
            .collect::<Vec<_>>()
    });
    assert_eq!(second, vec![(entity, false, false)]);

    let mut write_system =
        crate::scene::ecs::SystemState::<QueryState<Mut<'static, Health>>>::new(&mut world)
            .unwrap();
    write_system.run(&mut world, |mut query| {
        query.for_each_mut(|mut health| {
            assert!(health.is_added());
            assert!(health.is_changed());
            health.0 += 5;
        });
    });

    let changed = read_system.run(&mut world, |query| {
        query
            .iter()
            .map(|(entity, health)| (entity, health.0, health.is_changed()))
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![(entity, 15, true)]);
}

#[test]
fn system_query_iter_many_mut_preserves_order_duplicates_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Player)).unwrap();

    type ChangedHealth = QueryState<&'static mut Health, Changed<Health>>;
    let mut system = crate::scene::ecs::SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let requested = vec![marker_only, first, 999, first];

    let baseline = system.run(&mut world, |mut query| {
        let mut iter = query.iter_many_mut(&requested);
        let mut seen = Vec::new();
        while let Some(health) = iter.fetch_next() {
            seen.push(health.0);
            health.0 += 1;
        }
        seen
    });
    assert_eq!(baseline, vec![10, 11]);
    assert_eq!(world.get::<Health>(first), Some(&Health(12)));

    let unchanged = system.run(&mut world, |mut query| {
        query.iter_many_mut([first]).fetch_next().is_none()
    });
    assert!(unchanged);

    world.get_mut::<Health>(first).unwrap().0 = 20;
    let changed = system.run(&mut world, |mut query| {
        let mut iter = query.iter_many_mut([first]);
        let health = iter.fetch_next().unwrap();
        health.0 += 1;
        health.0
    });
    assert_eq!(changed, 21);
}

#[test]
fn query_state_cached_iteration_rebuilds_only_for_structural_changes() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let enemy = world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();

    let mut query = world.query_filtered::<(EntityId, &Health), Without<Enemy>>();
    let health_id = world.registered_component_id::<Health>().unwrap();
    assert_eq!(query.cache_rebuilds(), 1);
    assert_eq!(query.cached_entity_count(), 1);
    assert_eq!(query.cached_location_count(), 1);
    assert_eq!(
        query.cached_locations()[0],
        world.internal_entity_location(player).unwrap()
    );
    assert_eq!(
        query.cached_component_locations()[0].as_slice(),
        &[ComponentStorageLocation {
            component_id: health_id,
            storage_type: StorageType::Table,
            entity: world.internal_entity(player).unwrap(),
            table_row: Some(0),
        }]
    );
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
    assert_eq!(
        query
            .cached_locations()
            .iter()
            .map(|location| location.stable_id)
            .collect::<Vec<_>>(),
        vec![player, prop]
    );
    assert_eq!(
        query
            .cached_component_locations()
            .iter()
            .map(|locations| locations[0].entity)
            .collect::<Vec<_>>(),
        vec![
            world.internal_entity(player).unwrap(),
            world.internal_entity(prop).unwrap()
        ]
    );
    assert!(query.cached_revision() > initial_revision);

    world.remove::<Health>(player).unwrap();
    let after_remove = query
        .iter_cached(&world)
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(after_remove, vec![(prop, 2)]);
    assert_eq!(query.cache_rebuilds(), 3);
    assert_eq!(query.cached_location_count(), 1);
    assert_eq!(
        query.cached_locations()[0],
        world.internal_entity_location(prop).unwrap()
    );
    assert_eq!(query.cached_component_locations()[0][0].table_row, Some(0));
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
    assert_eq!(query.cache_rebuilds(), 2);

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
            .iter_many_cached_direct(&world, [player, enemy, enemy, 999])
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>(),
        vec![(enemy, 4), (enemy, 4)]
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
    assert_eq!(query.cache_rebuilds(), 3);

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
fn system_query_iter_many_cached_direct_preserves_order_duplicates_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Player)).unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = crate::scene::ecs::SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let requested = vec![marker_only, first, 999, first];

    let baseline = system.run(&mut world, |mut query| {
        query
            .iter_many_cached_direct(&requested)
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![(first, 10), (first, 10)]);

    let unchanged = system.run(&mut world, |mut query| {
        query
            .iter_many_cached_direct([first])
            .collect::<Vec<_>>()
            .is_empty()
    });
    assert!(unchanged);

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        query
            .iter_many_cached_direct([first])
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![(first, 11)]);
}

#[test]
fn query_state_cached_direct_iteration_reads_storage_locations() {
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
    assert_eq!(query.cached_component_locations().len(), 2);
    assert_eq!(query.cached_component_locations()[0][0].table_row, Some(0));
    assert_eq!(query.cached_component_locations()[1][0].table_row, Some(1));
    assert_eq!(query.cache_rebuilds(), 1);

    world.remove::<Health>(player).unwrap();
    let after_remove = query
        .iter_cached_direct(&world)
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();

    assert_eq!(after_remove, vec![(enemy, 4)]);
    assert_eq!(query.cache_rebuilds(), 2);
    assert_eq!(query.cached_component_locations()[0][0].table_row, Some(0));
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
fn query_state_cached_direct_iteration_reads_sparse_locations() {
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
    assert_eq!(
        query
            .cached_component_locations()
            .iter()
            .map(|locations| locations[0])
            .collect::<Vec<_>>(),
        vec![
            ComponentStorageLocation {
                component_id: score_id,
                storage_type: StorageType::SparseSet,
                entity: world.internal_entity(player).unwrap(),
                table_row: None,
            },
            ComponentStorageLocation {
                component_id: score_id,
                storage_type: StorageType::SparseSet,
                entity: world.internal_entity(enemy).unwrap(),
                table_row: None,
            },
        ]
    );

    world.remove::<SparseScore>(player).unwrap();
    let after_remove = query
        .iter_cached_direct(&world)
        .map(|(entity, score)| (entity, score.0))
        .collect::<Vec<_>>();

    assert_eq!(after_remove, vec![(enemy, 9)]);
    assert_eq!(
        query.cached_component_locations()[0][0],
        ComponentStorageLocation {
            component_id: score_id,
            storage_type: StorageType::SparseSet,
            entity: world.internal_entity(enemy).unwrap(),
            table_row: None,
        }
    );
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
