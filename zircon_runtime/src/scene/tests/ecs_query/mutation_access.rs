use super::*;

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
        cached_component_locations_for(&query, 0)[0].component_id,
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

    let error = match QueryState::<(&mut Health, &mut Health)>::try_new(&mut world) {
        Ok(_) => panic!("expected duplicate mutable component query to fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("mutably"));
}
