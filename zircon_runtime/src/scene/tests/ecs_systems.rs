use crate::scene::components::Name;
use crate::scene::ecs::{
    Added, Changed, CommandsParam, Component, EventReaderParam, EventWriterParam, LocalParam,
    ParamSet, QueryEntityError, QuerySingleError, QueryState, RemovedComponentsParam, ResMutParam,
    ResParam, Resource, SystemParamError, SystemState, With,
};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Player;

impl Component for Player {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

#[derive(Debug, PartialEq, Eq)]
struct Score(u32);

impl Resource for Score {}

#[derive(Debug, PartialEq, Eq)]
struct MissingScore(u32);

impl Resource for MissingScore {}

#[derive(Debug, PartialEq, Eq)]
struct HitEvent(u32);

#[derive(Default, Debug, PartialEq, Eq)]
struct LocalCounter(u32);

fn expect_query_error<T>(result: Result<T, QueryEntityError>) -> QueryEntityError {
    match result {
        Ok(_) => panic!("expected query error"),
        Err(error) => error,
    }
}

#[test]
fn commands_are_deferred_until_apply_deferred() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Queued".to_string()),)).unwrap();

    {
        let mut commands = world.commands();
        commands.entity(entity).insert((Health(7),));
        let spawned = commands
            .spawn((Name("Spawned".to_string()), Health(3)))
            .id();
        commands.insert_resource(Score(9));
        assert_eq!(spawned, 2);
    }

    assert!(world.get::<Health>(entity).is_none());
    assert!(world.get_resource::<Score>().is_none());
    assert_eq!(world.query::<&Health>().iter(&world).count(), 0);

    world.apply_deferred();

    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert_eq!(world.get_resource::<Score>(), Some(&Score(9)));
    assert_eq!(world.query::<&Health>().iter(&world).count(), 2);
}

#[test]
fn entity_commands_spawn_empty_and_entity_or_spawn_apply_in_queue_order() {
    let mut world = World::empty();
    let reserved = {
        let mut commands = world.commands();
        let reserved = commands.spawn_empty().id();
        commands
            .entity(reserved)
            .insert((Name("Reserved".to_string()), Health(1)));
        commands
            .entity_or_spawn(42)
            .insert((Name("Explicit".to_string()), Health(2)));
        reserved
    };

    assert!(!world.contains_entity(reserved));
    assert!(!world.contains_entity(42));

    world.apply_deferred();

    assert_eq!(world.get::<Name>(reserved).unwrap().0, "Reserved");
    assert_eq!(world.get::<Health>(reserved), Some(&Health(1)));
    assert_eq!(world.get::<Name>(42).unwrap().0, "Explicit");
    assert_eq!(world.get::<Health>(42), Some(&Health(2)));
}

#[test]
fn system_state_runs_query_resource_and_commands_params() {
    let mut world = World::empty();
    world.insert_resource(Score(1));
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let enemy = world.spawn((Name("Enemy".to_string()), Health(4))).unwrap();

    type Params = (
        QueryState<&'static mut Health, With<Player>>,
        ResMutParam<Score>,
        CommandsParam,
    );
    let mut system = SystemState::<Params>::new(&mut world).unwrap();

    system.run(&mut world, |(mut health_query, mut score, mut commands)| {
        health_query.for_each_mut(|health| health.0 += 2);
        score.0 += 1;
        commands.insert(enemy, Marker);
    });

    assert_eq!(world.get::<Health>(player), Some(&Health(12)));
    assert_eq!(world.get::<Health>(enemy), Some(&Health(4)));
    assert_eq!(world.get_resource::<Score>(), Some(&Score(2)));
    assert!(world.get::<Marker>(enemy).is_none());

    world.apply_deferred();

    assert_eq!(world.get::<Marker>(enemy), Some(&Marker));
}

#[test]
fn system_query_for_each_mut_uses_persistent_query_state_cache() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let enemy = world.spawn((Name("Enemy".to_string()), Health(4))).unwrap();

    type PlayerHealth = QueryState<&'static mut Health, With<Player>>;
    let mut system = SystemState::<PlayerHealth>::new(&mut world).unwrap();
    assert_eq!(system.state().cache_rebuilds(), 1);
    assert_eq!(system.state().cached_entity_count(), 1);

    system.run(&mut world, |mut query| {
        query.for_each_mut(|health| health.0 += 1);
    });

    assert_eq!(world.get::<Health>(player), Some(&Health(11)));
    assert_eq!(world.get::<Health>(enemy), Some(&Health(4)));
    assert_eq!(system.state().cache_rebuilds(), 1);

    let ally = world
        .spawn((Name("Ally".to_string()), Health(7), Player))
        .unwrap();
    system.run(&mut world, |mut query| {
        query.for_each_mut(|health| health.0 += 2);
    });

    assert_eq!(world.get::<Health>(player), Some(&Health(13)));
    assert_eq!(world.get::<Health>(ally), Some(&Health(9)));
    assert_eq!(world.get::<Health>(enemy), Some(&Health(4)));
    assert_eq!(system.state().cache_rebuilds(), 2);
    assert_eq!(system.state().cached_entity_count(), 2);
}

#[test]
fn system_query_get_mut_helpers_mutate_targets_and_reject_aliases() {
    let mut world = World::empty();
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let ally = world
        .spawn((Name("Ally".to_string()), Health(7), Player))
        .unwrap();
    let enemy = world.spawn((Name("Enemy".to_string()), Health(4))).unwrap();

    type PlayerHealth = QueryState<&'static mut Health, With<Player>>;
    let mut system = SystemState::<PlayerHealth>::new(&mut world).unwrap();
    system.run(&mut world, |mut query| {
        query.get_mut(player).unwrap().0 += 5;
        let healths = query.get_many_mut([ally, player]).unwrap();
        healths[0].0 += 2;
        healths[1].0 += 3;
    });

    assert_eq!(world.get::<Health>(player), Some(&Health(18)));
    assert_eq!(world.get::<Health>(ally), Some(&Health(9)));
    assert_eq!(world.get::<Health>(enemy), Some(&Health(4)));

    let errors = system.run(&mut world, |mut query| {
        (
            expect_query_error(query.get_mut(enemy)),
            expect_query_error(query.get_many_mut([player, player])),
            expect_query_error(query.get_many_mut([player, 999])),
        )
    });
    assert_eq!(
        errors,
        (
            QueryEntityError::QueryDoesNotMatch(enemy),
            QueryEntityError::AliasedMutability(player),
            QueryEntityError::NotSpawned(999),
        )
    );
}

#[test]
fn optional_resource_params_return_none_while_required_resources_error() {
    let mut world = World::empty();
    for error in [
        SystemState::<ResParam<MissingScore>>::new(&mut world).unwrap_err(),
        SystemState::<ResMutParam<MissingScore>>::new(&mut world).unwrap_err(),
    ] {
        assert_eq!(
            error,
            SystemParamError::MissingResource {
                type_name: std::any::type_name::<MissingScore>(),
            }
        );
    }

    type OptionalParams = (Option<ResParam<MissingScore>>, Option<ResMutParam<Score>>);
    let mut optional = SystemState::<OptionalParams>::new(&mut world).unwrap();
    let (missing_score, missing_mut_score) = optional.run(&mut world, |params| {
        (params.0.is_some(), params.1.is_some())
    });
    assert!(!missing_score);
    assert!(!missing_mut_score);

    world.insert_resource(Score(7));
    let mut optional_with_score =
        SystemState::<Option<ResMutParam<Score>>>::new(&mut world).unwrap();
    let value = optional_with_score.run(&mut world, |mut score| {
        let mut score = score.take().unwrap();
        score.0 += 1;
        score.0
    });
    assert_eq!(value, 8);
}

#[test]
fn param_set_allows_conflicting_params_with_segmented_access() {
    let mut world = World::empty();
    world.insert_resource(Score(1));

    type Params = ParamSet<(ResParam<Score>, ResMutParam<Score>)>;
    let mut system = SystemState::<Params>::new(&mut world).unwrap();
    let observed = system.run(&mut world, |mut params| {
        let before = params.p0().0;
        {
            let mut score = params.p1();
            score.0 += 4;
        }
        let after = params.p0().0;
        (before, after)
    });

    assert_eq!(observed, (1, 5));
    assert_eq!(world.resource::<Score>(), &Score(5));
}

#[test]
fn param_set_component_access_is_conservative_across_sibling_filters() {
    let mut world = World::empty();
    let marker_component = world.component_id::<Marker>();
    let health_component = world.component_id::<Health>();

    type Params = ParamSet<(
        QueryState<&'static mut Health, With<Marker>>,
        QueryState<&'static mut Health, crate::scene::ecs::Without<Marker>>,
    )>;
    let system = SystemState::<Params>::new(&mut world).unwrap();

    assert!(system
        .access()
        .component_access()
        .writes()
        .contains(&health_component));
    assert!(!system
        .access()
        .component_access()
        .with()
        .contains(&marker_component));
    assert!(!system
        .access()
        .component_access()
        .without()
        .contains(&marker_component));
}

#[test]
fn system_state_supports_tuple_params_up_to_eight_items() {
    let mut world = World::empty();
    type Params = (
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
    );
    let mut system = SystemState::<Params>::new(&mut world).unwrap();

    let observed = system.run(&mut world, |params| {
        let (mut p0, mut p1, mut p2, mut p3, mut p4, mut p5, mut p6, mut p7) = params;
        p0.0 = 1;
        p1.0 = 2;
        p2.0 = 3;
        p3.0 = 4;
        p4.0 = 5;
        p5.0 = 6;
        p6.0 = 7;
        p7.0 = 8;
        (p0.0, p1.0, p2.0, p3.0, p4.0, p5.0, p6.0, p7.0)
    });

    assert_eq!(observed, (1, 2, 3, 4, 5, 6, 7, 8));
}

#[test]
fn param_set_supports_segmented_access_up_to_eight_items() {
    let mut world = World::empty();
    type Params = ParamSet<(
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
        LocalParam<LocalCounter>,
    )>;
    let mut system = SystemState::<Params>::new(&mut world).unwrap();

    let observed = system.run(&mut world, |mut params| {
        let p0 = {
            let mut param = params.p0();
            param.0 = 1;
            param.0
        };
        let p1 = {
            let mut param = params.p1();
            param.0 = 2;
            param.0
        };
        let p2 = {
            let mut param = params.p2();
            param.0 = 3;
            param.0
        };
        let p3 = {
            let mut param = params.p3();
            param.0 = 4;
            param.0
        };
        let p4 = {
            let mut param = params.p4();
            param.0 = 5;
            param.0
        };
        let p5 = {
            let mut param = params.p5();
            param.0 = 6;
            param.0
        };
        let p6 = {
            let mut param = params.p6();
            param.0 = 7;
            param.0
        };
        let p7 = {
            let mut param = params.p7();
            param.0 = 8;
            param.0
        };
        (p0, p1, p2, p3, p4, p5, p6, p7)
    });

    assert_eq!(observed, (1, 2, 3, 4, 5, 6, 7, 8));
}

#[test]
fn event_reader_and_writer_use_current_and_next_queues() {
    let mut world = World::empty();
    type Writer = EventWriterParam<HitEvent>;
    type Reader = EventReaderParam<HitEvent>;

    let mut writer = SystemState::<Writer>::new(&mut world).unwrap();
    writer.run(&mut world, |mut events| events.send(HitEvent(3)));

    let mut reader = SystemState::<Reader>::new(&mut world).unwrap();
    let before_update = reader.run(&mut world, |events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert!(before_update.is_empty());

    world.update_events::<HitEvent>();
    let after_update = reader.run(&mut world, |events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert_eq!(after_update, vec![3]);
}

#[test]
fn added_and_changed_filters_use_system_run_windows() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();

    type AddedHealth = QueryState<(EntityId, &'static Health), Added<Health>>;
    let mut added_system = SystemState::<AddedHealth>::new(&mut world).unwrap();

    let first_added = added_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert_eq!(first_added, vec![first]);

    let second_added = added_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert!(second_added.is_empty());

    let second = world
        .spawn((Name("Second".to_string()), Health(1)))
        .unwrap();
    let new_added = added_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert_eq!(new_added, vec![second]);

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut changed_system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let baseline = changed_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![first, second]);

    let unchanged = changed_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert!(unchanged.is_empty());

    world.get_mut::<Health>(first).unwrap().0 += 5;
    let changed = changed_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![first]);
}

#[test]
fn system_query_cached_direct_rechecks_run_window_filters() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Tracked".to_string()), Health(10)))
        .unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let first = system.run(&mut world, |mut query| {
        query
            .iter_cached_direct()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(first, vec![(entity, 10)]);

    let second = system.run(&mut world, |mut query| {
        query
            .iter_cached_direct()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert!(second.is_empty());

    world.get_mut::<Health>(entity).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        query
            .iter_cached_direct()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![(entity, 11)]);
}

#[test]
fn system_query_iter_cached_reuses_state_and_rechecks_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    assert_eq!(system.state().cache_rebuilds(), 1);

    let baseline = system.run(&mut world, |mut query| {
        query
            .iter_cached()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![(first, 10)]);
    assert_eq!(system.state().cache_rebuilds(), 1);

    let unchanged = system.run(&mut world, |mut query| {
        query
            .iter_cached()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert!(unchanged.is_empty());
    assert_eq!(system.state().cache_rebuilds(), 1);

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        query
            .single_cached()
            .map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(changed, Ok((first, 11)));
    assert_eq!(system.state().cache_rebuilds(), 1);

    let second = world
        .spawn((Name("Second".to_string()), Health(1)))
        .unwrap();
    let after_spawn = system.run(&mut world, |mut query| {
        query
            .iter_cached()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(after_spawn, vec![(second, 1)]);
    assert_eq!(system.state().cache_rebuilds(), 2);
}

#[test]
fn system_query_count_and_empty_helpers_reuse_cache_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    assert_eq!(system.state().cache_rebuilds(), 1);

    let baseline = system.run(&mut world, |mut query| {
        (
            query.count(),
            query.is_empty(),
            query.count_cached(),
            query.is_empty_cached(),
            query.count_cached_direct(),
            query.is_empty_cached_direct(),
            query.contains(first),
            query.contains(marker_only),
            query.get(first).map(|(entity, health)| (entity, health.0)),
            query
                .get(marker_only)
                .map(|(entity, health)| (entity, health.0)),
            query.contains_cached(first),
            query.contains_cached(marker_only),
            query
                .get_cached(first)
                .map(|(entity, health)| (entity, health.0)),
            query
                .get_cached(marker_only)
                .map(|(entity, health)| (entity, health.0)),
            query.contains_cached_direct(first),
            query.contains_cached_direct(marker_only),
            query
                .get_cached_direct(first)
                .map(|(entity, health)| (entity, health.0)),
            query
                .get_cached_direct(marker_only)
                .map(|(entity, health)| (entity, health.0)),
        )
    });
    assert_eq!(
        (baseline.0, baseline.1, baseline.2, baseline.3, baseline.4, baseline.5),
        (1, false, 1, false, 1, false)
    );
    assert_eq!(
        (baseline.6, baseline.7, baseline.8, baseline.9),
        (
            true,
            false,
            Ok((first, 10)),
            Err(QueryEntityError::QueryDoesNotMatch(marker_only)),
        )
    );
    assert_eq!(
        (baseline.10, baseline.11, baseline.12, baseline.13),
        (
            true,
            false,
            Ok((first, 10)),
            Err(QueryEntityError::QueryDoesNotMatch(marker_only)),
        )
    );
    assert_eq!(
        (baseline.14, baseline.15, baseline.16, baseline.17),
        (
            true,
            false,
            Ok((first, 10)),
            Err(QueryEntityError::QueryDoesNotMatch(marker_only)),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    let unchanged = system.run(&mut world, |mut query| {
        (
            query.count(),
            query.is_empty(),
            query.count_cached(),
            query.is_empty_cached(),
            query.count_cached_direct(),
            query.is_empty_cached_direct(),
            query.contains(first),
            query.get(first).map(|(entity, health)| (entity, health.0)),
            query.contains_cached(first),
            query
                .get_cached(first)
                .map(|(entity, health)| (entity, health.0)),
            query.contains_cached_direct(first),
            query
                .get_cached_direct(first)
                .map(|(entity, health)| (entity, health.0)),
        )
    });
    assert_eq!(
        unchanged,
        (
            0,
            true,
            0,
            true,
            0,
            true,
            false,
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            false,
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            false,
            Err(QueryEntityError::QueryDoesNotMatch(first)),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        (
            query.count_cached(),
            query.is_empty_cached(),
            query.count_cached_direct(),
            query.is_empty_cached_direct(),
            query.contains_cached(first),
            query
                .get_cached(first)
                .map(|(entity, health)| (entity, health.0)),
            query.contains_cached_direct(first),
            query
                .get_cached_direct(first)
                .map(|(entity, health)| (entity, health.0)),
        )
    });
    assert_eq!(
        changed,
        (
            1,
            false,
            1,
            false,
            true,
            Ok((first, 11)),
            true,
            Ok((first, 11))
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    let second = world
        .spawn((Name("Second".to_string()), Health(1)))
        .unwrap();
    let after_spawn = system.run(&mut world, |mut query| {
        (
            query.count_cached(),
            query.is_empty_cached(),
            query.count_cached_direct(),
            query.is_empty_cached_direct(),
            query.contains_cached(first),
            query.contains_cached(second),
            query
                .get_cached(first)
                .map(|(entity, health)| (entity, health.0)),
            query
                .get_cached(second)
                .map(|(entity, health)| (entity, health.0)),
            query.contains_cached_direct(first),
            query.contains_cached_direct(second),
            query
                .get_cached_direct(first)
                .map(|(entity, health)| (entity, health.0)),
            query
                .get_cached_direct(second)
                .map(|(entity, health)| (entity, health.0)),
        )
    });
    assert_eq!(
        after_spawn,
        (
            1,
            false,
            1,
            false,
            false,
            true,
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            Ok((second, 1)),
            false,
            true,
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            Ok((second, 1)),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 2);
}

#[test]
fn system_query_get_many_helpers_preserve_order_duplicates_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        (
            query
                .get_many([first, first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many([first, marker_only])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached([first, first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached_direct([first, first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
        )
    });
    assert_eq!(
        baseline,
        (
            Ok([(first, 10), (first, 10)]),
            Err(QueryEntityError::QueryDoesNotMatch(marker_only)),
            Ok([(first, 10), (first, 10)]),
            Ok([(first, 10), (first, 10)]),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    let unchanged = system.run(&mut world, |mut query| {
        (
            query
                .get_many([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached_direct([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
        )
    });
    assert_eq!(
        unchanged,
        (
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            Err(QueryEntityError::QueryDoesNotMatch(first)),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        (
            query
                .get_many([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached_direct([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
        )
    });
    assert_eq!(
        changed,
        (Ok([(first, 11)]), Ok([(first, 11)]), Ok([(first, 11)]),)
    );
}

#[test]
fn system_query_iter_many_preserves_order_duplicates_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let requested = vec![marker_only, first, 999, first];
    let baseline = system.run(&mut world, |mut query| {
        (
            query
                .iter_many(&requested)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_cached(&requested)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
        )
    });
    assert_eq!(
        baseline,
        (
            vec![(first, 10), (first, 10)],
            vec![(first, 10), (first, 10)]
        )
    );

    let unchanged = system.run(&mut world, |mut query| {
        (
            query
                .iter_many([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_cached([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
        )
    });
    assert!(unchanged.0.is_empty());
    assert!(unchanged.1.is_empty());

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        (
            query
                .iter_many([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_cached([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
        )
    });
    assert_eq!(changed, (vec![(first, 11)], vec![(first, 11)]));
}

#[test]
fn system_query_single_helpers_report_zero_one_many_matches() {
    let mut world = World::empty();
    type PlayerHealth = QueryState<(EntityId, &'static Health), With<Player>>;
    let mut system = SystemState::<PlayerHealth>::new(&mut world).unwrap();

    let empty = system.run(&mut world, |query| {
        query.single().map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(empty, Err(QuerySingleError::NoEntities));

    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let one = system.run(&mut world, |query| {
        query.single().map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(one, Ok((player, 10)));

    let cached = system.run(&mut world, |mut query| {
        query
            .single_cached()
            .map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(cached, Ok((player, 10)));

    let cached_direct = system.run(&mut world, |mut query| {
        query
            .single_cached_direct()
            .map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(cached_direct, Ok((player, 10)));

    world
        .spawn((Name("Ally".to_string()), Health(7), Player))
        .unwrap();
    let many = system.run(&mut world, |query| {
        query.single().map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(many, Err(QuerySingleError::MultipleEntities));
}

#[test]
fn removed_components_reader_observes_direct_and_deferred_removals() {
    let mut world = World::empty();
    let direct = world
        .spawn((Name("Direct".to_string()), Health(1)))
        .unwrap();
    let deferred = world
        .spawn((Name("Deferred".to_string()), Health(2)))
        .unwrap();
    let despawned = world
        .spawn((Name("Despawned".to_string()), Health(3)))
        .unwrap();

    type RemovedHealth = RemovedComponentsParam<Health>;
    let mut system = SystemState::<RemovedHealth>::new(&mut world).unwrap();

    assert!(system
        .run(&mut world, |mut removed| removed.read().collect::<Vec<_>>())
        .is_empty());

    world.remove::<Health>(direct).unwrap();
    {
        let mut commands = world.commands();
        commands.entity(deferred).remove::<Health>();
        commands.entity(despawned).despawn();
    }

    let before_apply = system.run(&mut world, |mut removed| removed.read().collect::<Vec<_>>());
    assert_eq!(before_apply, vec![direct]);

    world.apply_deferred();

    let after_apply = system.run(&mut world, |mut removed| removed.read().collect::<Vec<_>>());
    assert_eq!(after_apply, vec![deferred, despawned]);
}

#[test]
fn local_param_state_persists_between_system_runs() {
    let mut world = World::empty();
    let mut system = SystemState::<LocalParam<LocalCounter>>::new(&mut world).unwrap();

    let first = system.run(&mut world, |mut counter| {
        counter.0 += 1;
        counter.0
    });
    let second = system.run(&mut world, |mut counter| {
        counter.0 += 1;
        counter.0
    });

    assert_eq!(first, 1);
    assert_eq!(second, 2);
}
