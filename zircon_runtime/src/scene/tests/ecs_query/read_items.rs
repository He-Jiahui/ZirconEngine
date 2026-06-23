use super::*;

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
