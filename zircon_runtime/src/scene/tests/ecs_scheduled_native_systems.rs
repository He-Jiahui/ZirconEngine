use std::sync::{Arc, Mutex};

use crate::scene::components::Name;
use crate::scene::ecs::{
    Added, Changed, Commands, CommandsParam, Component, Message, MessageReader, MessageReaderParam,
    Query, QueryState, RemovedComponents, RemovedComponentsParam, ScheduleConflictNodeKind,
    ScheduledSceneStep, ScheduledSceneStepRef, SystemStage, With,
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
    let step_labels =
        ScheduledSceneStep::iter_sorted_for_stage(SystemStage::Update, &[], &native_steps, &[])
            .map(|step| match step {
                ScheduledSceneStepRef::Native { id, .. } => format!("native:{id}"),
                ScheduledSceneStepRef::ApplyDeferred {
                    after_system_id, ..
                } => format!("apply_deferred:{after_system_id}"),
                ScheduledSceneStepRef::Internal(_) => "internal".to_string(),
                ScheduledSceneStepRef::Hook(_) => "hook".to_string(),
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
fn world_driver_reuses_tick_schedule_snapshots_for_stage_runs() {
    let driver_source = include_str!("../module/world_driver.rs");
    assert!(driver_source.contains("let hooks = core.scene_runtime_hook_stage_plan_snapshot();"));
    assert!(driver_source
        .contains("let schedule = level.with_world(|world| world.schedule().stage_plan());"));
    assert!(driver_source.contains("schedule.internal_systems_for_stage(*stage)"));
    assert!(driver_source.contains("schedule.native_steps_for_stage(*stage)"));
    assert!(driver_source.contains("hooks.hooks_for_stage(*stage)"));
    assert!(!driver_source.contains("world.schedule().systems().to_vec()"));
    assert!(!driver_source.contains("systems_for_stage(&systems"));
    assert!(!driver_source.contains("scene_runtime_hooks_for_stage(stage)"));

    let runtime_extensions_source = include_str!("../../core/runtime/handle/runtime_extensions.rs");
    assert!(runtime_extensions_source.contains("scene_runtime_hook_stage_plan_snapshot"));
    assert!(runtime_extensions_source.contains(") -> Arc<SceneRuntimeHookStagePlan>"));
    assert!(!runtime_extensions_source.contains("fn scene_runtime_hooks_snapshot("));

    let hook_state_source = include_str!("../../core/runtime/state/scene_runtime_hooks.rs");
    assert!(hook_state_source.contains("stage_plan: Arc<SceneRuntimeHookStagePlan>"));
    assert!(hook_state_source.contains("Arc::new(SceneRuntimeHookStagePlan::from_ordered"));
    assert!(hook_state_source.contains("Arc::clone(&self.stage_plan)"));
    assert!(hook_state_source
        .contains("fn from_ordered(ordered: &[SceneRuntimeHookRegistration]) -> Self"));
    assert!(hook_state_source.contains("let stage_hook_counts = hook_counts_by_stage(ordered);"));
    assert!(hook_state_source
        .contains("let mut by_stage = hook_groups_with_capacity(&stage_hook_counts);"));
    assert!(hook_state_source.contains("fn hook_counts_by_stage("));
    assert!(hook_state_source.contains("counts[hook.descriptor().stage.rank()] += 1;"));
    assert!(hook_state_source.contains("fn hook_groups_with_capacity("));
    assert!(hook_state_source.contains("Vec::with_capacity(stage_hook_counts[stage_index])"));
    assert!(!hook_state_source.contains(
        "ordered: Vec<SceneRuntimeHookRegistration>,\n    by_stage: [Vec<SceneRuntimeHookRegistration>; SystemStage::COUNT],"
    ));
    assert!(!hook_state_source.contains("let mut by_stage = empty_hook_groups();"));
    assert!(!hook_state_source.contains("by_stage: self.by_stage.clone()"));
    assert!(!hook_state_source
        .contains("#[derive(Clone, Debug)]\npub(crate) struct SceneRuntimeHookStagePlan"));

    let runner_source = include_str!("../ecs/schedule_runner.rs");
    assert!(runner_source.contains("internal_systems: &[SceneSystemDescriptor]"));
    assert!(runner_source.contains("native_steps: &[ScheduledSceneStep]"));
    assert!(runner_source.contains("hooks: &[SceneRuntimeHookRegistration]"));
    assert!(runner_source.contains("ScheduledSceneStep::iter_sorted_for_stage("));
    assert!(runner_source.contains("ScheduledSceneStepRef::Internal(system)"));
    assert!(!runner_source.contains("ScheduledSceneStep::sorted_for_stage("));
    assert!(!runner_source.contains("let steps ="));
    assert!(!runner_source.contains("scheduled_native_system_steps_for_stage(stage)"));

    let step_source = include_str!("../ecs/system/native/scheduled_scene_step.rs");
    assert!(step_source.contains("pub(crate) struct SortedScheduledSceneSteps<'a>"));
    assert!(step_source.contains("type Item = ScheduledSceneStepRef<'a>;"));
    assert!(step_source.contains("pub(crate) enum ScheduledSceneStepRef<'a>"));
    assert!(step_source.contains("fn compare_step_refs("));
    assert!(!step_source.contains("pub(crate) fn sorted_for_stage("));
    assert!(!step_source.contains("fn to_owned_step("));
    assert!(!step_source.contains("Internal(SceneSystemDescriptor)"));
    assert!(!step_source.contains("Hook(SceneRuntimeHookRegistration)"));
    assert!(step_source.contains("internal_systems: &[SceneSystemDescriptor]"));
    assert!(step_source.contains("native_steps: &[Self]"));
    assert!(step_source.contains("hooks: &[SceneRuntimeHookRegistration]"));
    assert!(step_source
        .contains("debug_assert!(internal_systems.iter().all(|system| system.stage == stage));"));
    assert!(step_source
        .contains("debug_assert!(hooks.iter().all(|hook| hook.descriptor().stage == stage));"));
    assert!(!step_source.contains("internal_systems: Vec<SceneSystemDescriptor>"));
    assert!(!step_source.contains("native_steps: Vec<Self>"));
    assert!(!step_source.contains("hooks: Vec<SceneRuntimeHookRegistration>"));
    assert!(!step_source.contains("filter(|system| system.stage == stage)"));
    assert!(!step_source.contains("filter(|hook| hook.descriptor().stage == stage)"));

    let schedule_source = include_str!("../ecs/schedule_stage_plan.rs");
    assert!(schedule_source.contains("struct SceneScheduleStagePlan"));
    assert!(schedule_source.contains("from_registry("));
    assert!(!schedule_source.contains("from_schedule("));
    assert!(schedule_source
        .contains("internal_systems_by_stage: [Vec<SceneSystemDescriptor>; SystemStage::COUNT]"));
    assert!(schedule_source
        .contains("native_steps_by_stage: [Vec<ScheduledSceneStep>; SystemStage::COUNT]"));
    assert!(schedule_source.contains("let systems = registry.systems();"));
    assert!(schedule_source
        .contains("let internal_system_counts = internal_system_counts_by_stage(systems);"));
    assert!(schedule_source.contains(
        "let mut internal_systems_by_stage =\n            internal_system_groups_with_capacity(&internal_system_counts);"
    ));
    assert!(schedule_source.contains("fn internal_system_counts_by_stage("));
    assert!(schedule_source.contains("counts[system.stage.rank()] += 1;"));
    assert!(schedule_source.contains("fn internal_system_groups_with_capacity("));
    assert!(schedule_source.contains("Vec::with_capacity(internal_system_counts[stage_index])"));
    assert!(!schedule_source.contains("fn empty_internal_system_groups("));
    assert!(!schedule_source.contains("std::array::from_fn(|_| Vec::new())"));

    let schedule_owner_source = include_str!("../ecs/schedule.rs");
    assert!(schedule_owner_source.contains("executor_plan: Arc<SceneScheduleStagePlan>"));
    assert!(schedule_owner_source.contains("Arc::clone(&self.executor_plan)"));
    assert!(schedule_owner_source.contains("fn refresh_executor_plan(&mut self)"));
    assert!(schedule_owner_source.contains("self.refresh_executor_plan();"));
    assert!(schedule_owner_source.contains("impl<'de> Deserialize<'de> for Schedule"));
    assert!(schedule_owner_source.contains("impl Clone for Schedule"));
    assert!(!schedule_owner_source.contains("pub stages: Vec<SystemStage>"));
    assert!(!schedule_owner_source.contains("SceneScheduleStagePlan::from_schedule(self)"));

    let registry_source = include_str!("../ecs/scene_system_registry.rs");
    assert!(registry_source.contains("native_system_steps_by_stage("));
    assert!(registry_source
        .contains("let native_step_counts = native_step_counts_by_stage(&self.native_systems);"));
    assert!(registry_source
        .contains("let mut by_stage = native_step_groups_with_capacity(&native_step_counts);"));
    assert!(registry_source.contains("fn native_step_counts_by_stage("));
    assert!(registry_source.contains("let step_count = if system.has_deferred_commands() {"));
    assert!(registry_source.contains("counts[system.stage().rank()] += step_count;"));
    assert!(registry_source.contains("fn native_step_groups_with_capacity("));
    assert!(registry_source.contains("Vec::with_capacity(native_step_counts[stage_index])"));
    assert!(!registry_source.contains("fn empty_native_step_groups("));
    assert!(!registry_source.contains("std::array::from_fn(|_| Vec::new())"));
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
