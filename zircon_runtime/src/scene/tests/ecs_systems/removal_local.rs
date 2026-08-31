use super::*;
use crate::scene::ecs::{
    InternalSceneSystem, RemovedComponentEvents, RemovedComponentReader, RemovedComponentRetention,
};

#[test]
fn removed_component_write_receipt_reports_byte_budget_rejection() {
    let mut events = RemovedComponentEvents::default();
    events.configure_retention::<Health>(RemovedComponentRetention::new(usize::MAX, 1, u64::MAX));

    let receipt = events.push::<Health>(1);

    assert_eq!(receipt.sequence(), Some(0));
    assert!(!receipt.is_retained());
    assert_eq!(receipt.dropped_entries(), 1);
    assert!(receipt.dropped_bytes() > 0);
}

#[test]
fn removed_component_reader_reports_budgeted_slow_reader_gap() {
    let mut world = World::empty();
    world.configure_removed_component_retention::<Health>(RemovedComponentRetention::new(
        2,
        usize::MAX,
        u64::MAX,
    ));
    let mut reader = RemovedComponentReader::<Health>::default();

    let first = world.spawn((Name("First".to_string()), Health(1))).unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(2)))
        .unwrap();
    let third = world.spawn((Name("Third".to_string()), Health(3))).unwrap();
    world.remove::<Health>(first).unwrap();
    world.remove::<Health>(second).unwrap();
    world.remove::<Health>(third).unwrap();

    let metrics = world
        .removed_component_retention_metrics::<Health>()
        .unwrap();
    assert_eq!(metrics.retained_entries, 2);
    assert_eq!(metrics.budget_dropped_entries, 1);

    let observed = reader
        .read(world.removed_component_events())
        .collect::<Vec<_>>();
    assert_eq!(observed, vec![second, third]);
    assert_eq!(reader.dropped_count(), 1);
}

#[test]
fn removed_component_update_events_reclaims_expired_entries_without_reader() {
    let mut world = World::empty();
    world.configure_removed_component_retention::<Health>(RemovedComponentRetention::new(
        usize::MAX,
        usize::MAX,
        0,
    ));
    let entity = world
        .spawn((Name("Expired".to_string()), Health(1)))
        .unwrap();
    world.remove::<Health>(entity).unwrap();

    world.run_internal_scene_system(InternalSceneSystem::UpdateEvents);

    let mut reader = RemovedComponentReader::<Health>::default();
    assert!(
        reader
            .read(world.removed_component_events())
            .next()
            .is_none()
    );
    let metrics = world
        .removed_component_retention_metrics::<Health>()
        .unwrap();
    assert_eq!(metrics.age_dropped_entries, 1);
}

#[test]
fn removed_component_clear_trackers_reclaims_expired_entries() {
    let mut world = World::empty();
    world.configure_removed_component_retention::<Health>(RemovedComponentRetention::new(
        usize::MAX,
        usize::MAX,
        0,
    ));
    let entity = world
        .spawn((Name("ClearTrackers".to_string()), Health(1)))
        .unwrap();
    world.remove::<Health>(entity).unwrap();

    world.clear_trackers();

    let mut reader = RemovedComponentReader::<Health>::default();
    assert!(
        reader
            .read(world.removed_component_events())
            .next()
            .is_none()
    );
    assert_eq!(
        world
            .removed_component_retention_metrics::<Health>()
            .unwrap()
            .age_dropped_entries,
        1
    );
}

#[test]
fn removed_component_readers_independently_report_the_same_retention_gap() {
    let mut world = World::empty();
    world.configure_removed_component_retention::<Health>(RemovedComponentRetention::new(
        2,
        usize::MAX,
        u64::MAX,
    ));
    let mut first_reader = RemovedComponentReader::<Health>::default();
    let mut second_reader = RemovedComponentReader::<Health>::default();

    let first = world
        .spawn((Name("FirstReader".to_string()), Health(1)))
        .unwrap();
    let second = world
        .spawn((Name("SecondReader".to_string()), Health(2)))
        .unwrap();
    let third = world
        .spawn((Name("ThirdReader".to_string()), Health(3)))
        .unwrap();
    world.remove::<Health>(first).unwrap();
    world.remove::<Health>(second).unwrap();
    world.remove::<Health>(third).unwrap();

    assert_eq!(
        first_reader
            .read(world.removed_component_events())
            .collect::<Vec<_>>(),
        vec![second, third]
    );
    assert_eq!(
        second_reader
            .read(world.removed_component_events())
            .collect::<Vec<_>>(),
        vec![second, third]
    );
    assert_eq!(first_reader.dropped_count(), 1);
    assert_eq!(second_reader.dropped_count(), 1);
}

#[test]
fn removed_component_reader_keeps_unconsumed_entries_after_iterator_drop() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("IteratorFirst".to_string()), Health(1)))
        .unwrap();
    let second = world
        .spawn((Name("IteratorSecond".to_string()), Health(2)))
        .unwrap();
    world.remove::<Health>(first).unwrap();
    world.remove::<Health>(second).unwrap();
    let mut reader = RemovedComponentReader::<Health>::default();

    {
        let mut iterator = reader.read(world.removed_component_events());
        assert_eq!(iterator.next(), Some(first));
    }

    assert_eq!(
        reader
            .read(world.removed_component_events())
            .collect::<Vec<_>>(),
        vec![second]
    );
}

#[test]
fn removed_component_explicit_clear_does_not_count_as_reader_lag() {
    let mut world = World::empty();
    let retired = world
        .spawn((Name("Retired".to_string()), Health(1)))
        .unwrap();
    world.remove::<Health>(retired).unwrap();

    let mut reader = RemovedComponentReader::<Health>::default();
    world.clear_removed_component_events::<Health>();
    assert!(
        reader
            .read(world.removed_component_events())
            .next()
            .is_none()
    );

    let current = world
        .spawn((Name("Current".to_string()), Health(2)))
        .unwrap();
    world.remove::<Health>(current).unwrap();
    assert_eq!(
        reader
            .read(world.removed_component_events())
            .collect::<Vec<_>>(),
        vec![current]
    );
    assert_eq!(reader.dropped_count(), 0);
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

    assert!(
        system
            .run(&mut world, |mut removed| removed.read().collect::<Vec<_>>())
            .is_empty()
    );

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

#[test]
fn scheduled_native_system_keeps_local_state_between_ticks() {
    use std::sync::{Arc, Mutex};

    let mut world = World::empty();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let system_observed = observed.clone();
    world
        .register_native_system::<LocalParam<LocalCounter>, _>(
            "gameplay.local-counter",
            SystemStage::Update,
            0,
            move |mut counter: Local<'_, LocalCounter>| {
                counter.0 += 1;
                system_observed.lock().unwrap().push(counter.0);
            },
        )
        .unwrap();

    world.run_native_scene_systems_for_stage(SystemStage::Update);
    world.run_native_scene_systems_for_stage(SystemStage::Update);

    assert_eq!(*observed.lock().unwrap(), vec![1, 2]);
}
