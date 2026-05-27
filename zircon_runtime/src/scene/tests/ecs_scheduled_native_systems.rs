use std::sync::{Arc, Mutex};

use crate::scene::components::Name;
use crate::scene::ecs::{
    Added, Changed, Commands, CommandsParam, Component, Message, MessageReader, MessageReaderParam,
    Query, QueryState, RemovedComponents, RemovedComponentsParam, ScheduleConflictNodeKind,
    ScheduledSceneStep, SystemStage, With,
};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

#[derive(Debug, PartialEq, Eq)]
struct HitMessage(u32);

impl Message for HitMessage {}

#[test]
fn scheduled_native_system_uses_added_and_changed_windows() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let observed_changed = Arc::new(Mutex::new(Vec::new()));
    let observed_added = Arc::new(Mutex::new(Vec::new()));

    {
        let observed = observed_changed.clone();
        world
            .register_native_system::<QueryState<(EntityId, &'static Health), Changed<Health>>, _>(
                "gameplay.changed-health",
                SystemStage::Update,
                0,
                move |query: Query<'_, (EntityId, &'static Health), Changed<Health>>| {
                    observed
                        .lock()
                        .unwrap()
                        .push(query.iter().map(|(entity, _)| entity).collect::<Vec<_>>());
                },
            )
            .unwrap();
    }

    {
        let observed = observed_added.clone();
        world
            .register_native_system::<QueryState<(EntityId, &'static Health), Added<Health>>, _>(
                "gameplay.added-health",
                SystemStage::Update,
                1,
                move |query: Query<'_, (EntityId, &'static Health), Added<Health>>| {
                    observed
                        .lock()
                        .unwrap()
                        .push(query.iter().map(|(entity, _)| entity).collect::<Vec<_>>());
                },
            )
            .unwrap();
    }

    world.run_native_scene_systems_for_stage(SystemStage::Update);
    world.run_native_scene_systems_for_stage(SystemStage::Update);
    world.get_mut::<Health>(first).unwrap().0 += 5;
    world.run_native_scene_systems_for_stage(SystemStage::Update);

    assert_eq!(
        *observed_changed.lock().unwrap(),
        vec![vec![first], vec![], vec![first]]
    );
    assert_eq!(
        *observed_added.lock().unwrap(),
        vec![vec![first], vec![], vec![]]
    );
}

#[test]
fn scheduled_native_message_reader_keeps_cursor() {
    let mut world = World::empty();
    let observed = Arc::new(Mutex::new(Vec::new()));

    {
        let observed = observed.clone();
        world
            .register_native_system::<MessageReaderParam<HitMessage>, _>(
                "gameplay.hit-reader",
                SystemStage::Update,
                0,
                move |mut messages: MessageReader<'_, HitMessage>| {
                    observed.lock().unwrap().push(
                        messages
                            .read()
                            .map(|(_id, message)| message.0)
                            .collect::<Vec<_>>(),
                    );
                },
            )
            .unwrap();
    }

    world.send_message(HitMessage(1));
    world.run_native_scene_systems_for_stage(SystemStage::Update);
    world.run_native_scene_systems_for_stage(SystemStage::Update);
    world.send_message(HitMessage(2));
    world.run_native_scene_systems_for_stage(SystemStage::Update);

    assert_eq!(*observed.lock().unwrap(), vec![vec![1], vec![], vec![2]]);
}

#[test]
fn scheduled_native_removed_components_reader_keeps_cursor() {
    let mut world = World::empty();
    let first = world.spawn((Name("First".to_string()), Health(1))).unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(2)))
        .unwrap();
    let observed = Arc::new(Mutex::new(Vec::new()));

    {
        let observed = observed.clone();
        world
            .register_native_system::<RemovedComponentsParam<Health>, _>(
                "gameplay.removed-health-reader",
                SystemStage::Update,
                0,
                move |mut removed: RemovedComponents<'_, Health>| {
                    observed
                        .lock()
                        .unwrap()
                        .push(removed.read().collect::<Vec<_>>());
                },
            )
            .unwrap();
    }

    world.run_native_scene_systems_for_stage(SystemStage::Update);
    world.remove::<Health>(first).unwrap();
    world.run_native_scene_systems_for_stage(SystemStage::Update);
    world.run_native_scene_systems_for_stage(SystemStage::Update);
    world.remove::<Health>(second).unwrap();
    world.run_native_scene_systems_for_stage(SystemStage::Update);

    assert_eq!(
        *observed.lock().unwrap(),
        vec![vec![], vec![first], vec![], vec![second]]
    );
}

#[test]
fn scheduled_native_commands_flush_before_later_ordered_systems() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Target".to_string()),)).unwrap();
    let observed = Arc::new(Mutex::new(Vec::new()));

    world
        .register_native_system::<(CommandsParam, QueryState<(), With<Marker>>), _>(
            "gameplay.insert-marker",
            SystemStage::Update,
            0,
            {
                let observed = observed.clone();
                move |(mut commands, query): (Commands<'_>, Query<'_, (), With<Marker>>)| {
                    observed.lock().unwrap().push(!query.is_empty());
                    commands.entity(entity).insert((Marker,));
                    observed.lock().unwrap().push(!query.is_empty());
                }
            },
        )
        .unwrap();

    {
        let observed = observed.clone();
        world
            .register_native_system::<QueryState<(), With<Marker>>, _>(
                "gameplay.observe-marker",
                SystemStage::Update,
                1,
                move |query: Query<'_, (), With<Marker>>| {
                    observed.lock().unwrap().push(!query.is_empty());
                },
            )
            .unwrap();
    }

    world.run_native_scene_systems_for_stage(SystemStage::Update);

    assert_eq!(*observed.lock().unwrap(), vec![false, false, true]);
}

#[test]
fn scheduled_native_steps_show_apply_deferred_after_command_systems() {
    let mut world = World::empty();

    world
        .register_native_system::<(), _>("gameplay.read-only", SystemStage::Update, -1, |()| {})
        .unwrap();
    world
        .register_native_system::<CommandsParam, _>(
            "gameplay.commands",
            SystemStage::Update,
            0,
            |_: Commands<'_>| {},
        )
        .unwrap();
    world
        .register_native_system::<(), _>("gameplay.after-commands", SystemStage::Update, 1, |()| {})
        .unwrap();

    let native_steps = world.scheduled_native_system_steps_for_stage(SystemStage::Update);
    let step_labels = ScheduledSceneStep::sorted_for_stage(
        SystemStage::Update,
        Vec::new(),
        native_steps,
        Vec::new(),
    )
    .into_iter()
    .map(|step| match step {
        ScheduledSceneStep::Native { id, .. } => format!("native:{id}"),
        ScheduledSceneStep::ApplyDeferred {
            after_system_id, ..
        } => format!("apply_deferred:{after_system_id}"),
        ScheduledSceneStep::Internal(_) => "internal".to_string(),
        ScheduledSceneStep::Hook(_) => "hook".to_string(),
    })
    .collect::<Vec<_>>();

    assert_eq!(
        step_labels,
        vec![
            "native:gameplay.read-only",
            "native:gameplay.commands",
            "apply_deferred:gameplay.commands",
            "native:gameplay.after-commands",
        ]
    );
}

#[test]
fn scheduled_native_conflict_graph_keeps_apply_deferred_as_barrier_batch() {
    let mut world = World::empty();
    world
        .spawn((Name("Observed".to_string()), Health(10)))
        .unwrap();

    world
        .register_native_system::<QueryState<&'static Health>, _>(
            "gameplay.read-health",
            SystemStage::Update,
            -1,
            |_: Query<'_, &'static Health>| {},
        )
        .unwrap();
    world
        .register_native_system::<CommandsParam, _>(
            "gameplay.commands",
            SystemStage::Update,
            0,
            |_: Commands<'_>| {},
        )
        .unwrap();
    world
        .register_native_system::<QueryState<&'static Health>, _>(
            "gameplay.after-commands",
            SystemStage::Update,
            1,
            |_: Query<'_, &'static Health>| {},
        )
        .unwrap();

    let graph = world
        .schedule()
        .native_system_conflict_graph_for_stage(SystemStage::Update);
    let node_labels = graph
        .nodes()
        .iter()
        .map(|node| {
            let kind = match node.kind() {
                ScheduleConflictNodeKind::System => "system",
                ScheduleConflictNodeKind::Barrier => "barrier",
            };
            format!("{kind}:{}", node.system_id())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        node_labels,
        vec![
            "system:gameplay.read-health",
            "system:gameplay.commands",
            "barrier:apply_deferred:gameplay.commands",
            "system:gameplay.after-commands",
        ]
    );
    assert!(graph.edges().is_empty());

    let batches = graph.conservative_parallel_batches();
    let batch_labels = batches
        .iter()
        .map(|batch| {
            (
                batch.has_barrier(),
                batch
                    .system_ids()
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        batch_labels,
        vec![
            (false, vec!["gameplay.read-health", "gameplay.commands"]),
            (true, vec!["apply_deferred:gameplay.commands"]),
            (false, vec!["gameplay.after-commands"]),
        ]
    );
}
