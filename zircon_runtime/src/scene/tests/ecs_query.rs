use crate::scene::components::{Name, RenderLayerMask};
use crate::scene::ecs::{Component, Mut, QueryState, Ref, With, Without};
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
fn query_state_mutates_matching_components_without_touching_filtered_entities() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let enemy = world
        .spawn((Name("Enemy".to_string()), Health(4), Enemy))
        .unwrap();

    let mut query = world.query_filtered::<&mut Health, With<Player>>();
    query.for_each_mut(&mut world, |health| health.0 += 5);

    assert_eq!(world.get::<Health>(player), Some(&Health(15)));
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
