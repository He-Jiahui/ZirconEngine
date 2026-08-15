use super::*;

#[derive(Debug, Eq, PartialEq)]
struct TypedWorkerHealth(u32);

impl Component for TypedWorkerHealth {}

#[derive(Debug, Eq, PartialEq)]
struct TypedWorkerMarker;

impl Component for TypedWorkerMarker {}

#[derive(Debug, Eq, PartialEq)]
struct TypedWorkerStructuralSnapshot {
    names: Vec<String>,
    resolved_entities: Vec<EntityId>,
    alpha_has_health: bool,
    alpha_has_marker: bool,
    marker_events: Vec<LifecycleEventKind>,
    error_operations: Vec<DeferredCommandOperation>,
    error_targets: Vec<DeferredCommandTarget>,
    rejected_spawn_resolved: bool,
}

#[test]
fn typed_worker_structural_commands_are_deterministic_at_1_8_and_64_workers() {
    let serial = typed_worker_structural_snapshot(1);
    let medium = typed_worker_structural_snapshot(8);
    let wide = typed_worker_structural_snapshot(64);

    assert_eq!(serial.names.as_slice(), ["alpha", "beta", "zeta"],);
    assert!(!serial.alpha_has_health);
    assert!(serial.alpha_has_marker);
    assert_eq!(
        serial.marker_events.as_slice(),
        &[LifecycleEventKind::Add, LifecycleEventKind::Insert]
    );
    assert_eq!(
        serial.error_operations.as_slice(),
        &[
            DeferredCommandOperation::InsertBundle,
            DeferredCommandOperation::InsertBundle,
        ]
    );
    assert_ne!(serial.error_targets[0], serial.error_targets[1]);
    assert!(!serial.rejected_spawn_resolved);
    assert_eq!(medium, serial);
    assert_eq!(wide, serial);
}
fn typed_worker_structural_snapshot(parallelism: usize) -> TypedWorkerStructuralSnapshot {
    let mut world = World::empty();
    let alpha = Arc::new(Mutex::new(None));
    let beta = Arc::new(Mutex::new(None));
    let zeta = Arc::new(Mutex::new(None));
    let rejected = Arc::new(Mutex::new(Vec::new()));
    let marker_events = Arc::new(Mutex::new(Vec::new()));
    for kind in [LifecycleEventKind::Add, LifecycleEventKind::Insert] {
        let marker_events = Arc::clone(&marker_events);
        world.observe_component_lifecycle::<TypedWorkerMarker>(kind, move |_world, _event| {
            marker_events
                .lock()
                .expect("typed worker marker lifecycle events")
                .push(kind);
        });
    }
    let zeta_handle = Arc::clone(&zeta);
    world
        .register_worldless_native_system::<CommandsParam, _>(
            "tests.typed_worker.zeta",
            SystemStage::Update,
            30,
            move |mut commands| {
                *zeta_handle.lock().expect("zeta deferred handle") = Some(
                    commands
                        .spawn((Name("zeta".to_string()),))
                        .into_deferred_entity(),
                );
                commands.queue_fn(|_world| {});
            },
        )
        .unwrap();
    let beta_handle = Arc::clone(&beta);
    world
        .register_worldless_native_system::<CommandsParam, _>(
            "tests.typed_worker.beta",
            SystemStage::Update,
            10,
            move |mut commands| {
                *beta_handle.lock().expect("beta deferred handle") = Some(
                    commands
                        .spawn((Name("beta".to_string()),))
                        .into_deferred_entity(),
                );
                commands.queue_fn(|_world| {});
            },
        )
        .unwrap();
    let alpha_handle = Arc::clone(&alpha);
    world
        .register_worldless_native_system::<CommandsParam, _>(
            "tests.typed_worker.alpha",
            SystemStage::Update,
            10,
            move |mut commands| {
                let mut entity = commands.spawn_empty();
                entity.insert((
                    Name("alpha".to_string()),
                    TypedWorkerHealth(1),
                    TypedWorkerMarker,
                ));
                entity.remove::<TypedWorkerHealth>();
                *alpha_handle.lock().expect("alpha deferred handle") =
                    Some(entity.into_deferred_entity());
                commands.queue_fn(|_world| {});
            },
        )
        .unwrap();
    let rejected_handle = Arc::clone(&rejected);
    world
        .register_worldless_native_system::<CommandsParam, _>(
            "tests.typed_worker.rejected",
            SystemStage::Update,
            40,
            move |mut commands| {
                for value in [3, 4] {
                    let mut entity = commands.spawn_empty();
                    entity.insert((TypedWorkerHealth(value), TypedWorkerHealth(value + 10)));
                    rejected_handle
                        .lock()
                        .expect("rejected deferred handles")
                        .push(entity.into_deferred_entity());
                }
            },
        )
        .unwrap();

    let dispatches = world
        .scheduled_native_system_steps_for_stage(SystemStage::Update)
        .into_iter()
        .filter_map(|step| match step {
            ScheduledSceneStep::Native {
                id,
                stage,
                order,
                worker_safe: true,
                ..
            } => Some((
                id.clone(),
                DeferredSystemKey::compiled(stage.rank(), order, id),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    let ids = dispatches
        .iter()
        .map(|(id, _)| id.as_str())
        .collect::<Vec<_>>();
    let mut systems = world
        .take_worldless_native_scene_systems(&ids)
        .expect("typed worker systems must remain registered");
    for (system, (_, key)) in systems.iter_mut().zip(dispatches.iter()) {
        system
            .worker_command_buffer_mut()
            .expect("typed CommandsParam owns one worker command buffer")
            .bind_compiled_key(key.clone());
    }

    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(parallelism),
    ));
    let mut timings = vec![NativeSystemCallbackTiming::default(); systems.len()];
    run_worldless_systems(&scheduler, &mut systems, &mut timings, Instant::now());

    let mut buffers = systems
        .iter_mut()
        .filter_map(|system| system.worker_command_buffer_mut())
        .collect::<Vec<_>>();
    world
        .merge_worker_command_buffers(&mut buffers)
        .expect("compiled typed worker keys must remain unique");
    let report = world.apply_deferred();
    world.reclaim_worker_command_buffers(&mut buffers);
    world.restore_worldless_native_scene_systems(systems);

    assert_eq!(report.applied_count(), 12);
    assert_eq!(report.error_count(), 2);
    let resolved_entities = [&alpha, &beta, &zeta]
        .into_iter()
        .map(|handle| {
            let handle = handle
                .lock()
                .expect("typed worker deferred handle")
                .as_ref()
                .expect("typed worker spawn must record its deferred handle")
                .clone();
            report
                .resolve(&handle)
                .expect("typed worker spawn must resolve at the barrier")
        })
        .collect();
    let query = world.query::<&Name>();
    let rejected_spawns = rejected.lock().expect("rejected deferred handles").clone();
    assert_eq!(rejected_spawns.len(), 2);
    TypedWorkerStructuralSnapshot {
        names: query.iter(&world).map(|name| name.0.clone()).collect(),
        alpha_has_health: world
            .get::<TypedWorkerHealth>(resolved_entities[0])
            .is_some(),
        alpha_has_marker: world
            .get::<TypedWorkerMarker>(resolved_entities[0])
            .is_some(),
        resolved_entities,
        marker_events: marker_events
            .lock()
            .expect("typed worker marker lifecycle events")
            .clone(),
        error_operations: report
            .errors()
            .iter()
            .map(|error| error.operation())
            .collect(),
        error_targets: report
            .errors()
            .iter()
            .map(|error| error.target().clone())
            .collect(),
        rejected_spawn_resolved: rejected_spawns
            .iter()
            .any(|spawn| report.resolve(spawn).is_some()),
    }
}
