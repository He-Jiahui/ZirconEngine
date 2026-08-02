use super::*;

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
