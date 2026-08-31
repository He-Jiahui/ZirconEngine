use std::sync::{Arc, Mutex};
use std::{fs, path::Path};

use crate::scene::components::Name;
use crate::scene::ecs::{
    Added, Changed, Commands, CommandsParam, Component, Local, LocalParam, Message, MessageReader,
    MessageReaderParam, Query, QueryState, RemovedComponents, RemovedComponentsParam,
    ScheduleConflictNodeKind, ScheduledSceneStep, ScheduledSceneStepRef, SystemStage, With,
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
fn scene_hook_hard_cut_routes_script_updates_through_runtime_scene_systems() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repository_root = runtime_root
        .parent()
        .expect("zircon_runtime must live below the repository root");
    let runtime_source = runtime_root.join("src");
    let script_system_path = runtime_source.join("script/vm/scene_system.rs");
    let script_system_source = fs::read_to_string(&script_system_path)
        .expect("script scene execution must live in scene_system.rs");

    assert!(!runtime_source.join("scene/runtime_hook").exists());
    assert!(!runtime_source.join("script/vm/scene_hook.rs").exists());
    assert!(script_system_source.contains("RuntimeSceneSystemContext"));
    assert!(script_system_source.contains("SCRIPT_SCENE_FIXED_UPDATE_SYSTEM"));
    assert!(script_system_source.contains("SCRIPT_SCENE_UPDATE_SYSTEM"));
    assert!(!script_system_source.contains("SceneRuntimeHook"));

    for relative_path in [
        "scene/mod.rs",
        "scene/module/mod.rs",
        "scene/module/world_driver.rs",
        "scene/ecs/schedule_runner.rs",
        "scene/ecs/system/native/scheduled_scene_step.rs",
        "plugin/extension_registry/runtime_extension_registry.rs",
        "dynamic_api/session/construction.rs",
    ] {
        let source = fs::read_to_string(runtime_source.join(relative_path))
            .unwrap_or_else(|error| panic!("read {relative_path}: {error}"));
        assert!(
            !source.contains("SceneRuntimeHook") && !source.contains("scene_hook"),
            "{relative_path} retains the superseded scene-hook path"
        );
    }

    let sdk_registration =
        fs::read_to_string(repository_root.join("zircon_plugins/plugin_sdk/src/registration.rs"))
            .expect("read plugin SDK registration surface");
    assert!(!sdk_registration.contains("scene_hook"));
    assert!(!sdk_registration.contains("SceneRuntimeHook"));

    let vm_plugin = fs::read_to_string(
        repository_root.join("zircon_plugins/zr_vm_language/runtime/src/plugin.rs"),
    )
    .expect("read VM language runtime plugin registration");
    assert!(vm_plugin.contains("ScriptSceneRuntimeSystem::fixed_update()"));
    assert!(vm_plugin.contains("ScriptSceneRuntimeSystem::update()"));
    assert!(vm_plugin.contains(".runtime_scene_system(id, stage"));
    assert!(!vm_plugin.contains(".scene_hook("));
}

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
                move |mut query: Query<'_, (EntityId, &'static Health), Changed<Health>>| {
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
                move |mut query: Query<'_, (EntityId, &'static Health), Added<Health>>| {
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
                move |(mut commands, mut query): (Commands<'_>, Query<'_, (), With<Marker>>)| {
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
                move |mut query: Query<'_, (), With<Marker>>| {
                    observed.lock().unwrap().push(!query.is_empty());
                },
            )
            .unwrap();
    }

    world.run_native_scene_systems_for_stage(SystemStage::Update);

    assert_eq!(*observed.lock().unwrap(), vec![false, false, true]);
}

#[test]
fn explicit_worldless_native_system_uses_its_local_command_lane_at_the_stage_barrier() {
    let mut world = World::empty();
    let observed_local_runs = Arc::new(Mutex::new(Vec::new()));
    let observed_local_runs_for_system = Arc::clone(&observed_local_runs);

    world
        .register_worldless_native_system::<(CommandsParam, LocalParam<u32>), _>(
            "gameplay.worldless-commands",
            SystemStage::Update,
            0,
            move |(mut commands, mut local): (Commands<'_>, Local<'_, u32>)| {
                *local += 1;
                observed_local_runs_for_system.lock().unwrap().push(*local);
                commands.spawn((Name(format!("worldless-{}", *local)),));
            },
        )
        .expect("only the explicit worldless path may construct a WorkerSafe typed system");

    assert!(matches!(
        world
            .scheduled_native_system_steps_for_stage(SystemStage::Update)
            .as_slice(),
        [ScheduledSceneStep::Native {
            worker_safe: true,
            ..
        }]
    ));

    world.run_native_scene_systems_for_stage(SystemStage::Update);
    world.run_native_scene_systems_for_stage(SystemStage::Update);

    assert_eq!(*observed_local_runs.lock().unwrap(), vec![1, 2]);
    assert_eq!(world.query::<&Name>().iter(&world).count(), 2);
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
        ScheduledSceneStep::iter_sorted_for_stage(SystemStage::Update, &[], &native_steps)
            .map(|step| match step {
                ScheduledSceneStepRef::Native { id, .. } => format!("native:{id}"),
                ScheduledSceneStepRef::Runtime { id, .. } => format!("runtime:{id}"),
                ScheduledSceneStepRef::ApplyDeferred {
                    after_system_id, ..
                } => format!("apply_deferred:{after_system_id}"),
                ScheduledSceneStepRef::Internal(_) => "internal".to_string(),
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
    assert!(!driver_source.contains("SceneRuntimeHook"));
    assert!(!driver_source.contains("scene_runtime_hook"));
    assert!(
        driver_source
            .contains("let schedule = level.with_world(|world| world.schedule().stage_plan());")
    );
    assert!(driver_source.contains("schedule.internal_systems_for_stage(stage)"));
    assert!(driver_source.contains("schedule.native_steps_for_stage(stage)"));
    assert!(!driver_source.contains("world.schedule().systems().to_vec()"));
    assert!(!driver_source.contains("systems_for_stage(&systems"));
    assert!(!driver_source.contains("scene_runtime_hooks_for_stage(stage)"));

    let runtime_extensions_source = include_str!("../../core/runtime/handle/runtime_extensions.rs");
    assert!(!runtime_extensions_source.contains("SceneRuntimeHook"));
    assert!(!runtime_extensions_source.contains("scene_runtime_hook_stage_plan_snapshot"));

    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/scene/runtime_hook")
            .exists()
    );

    let runner_source = include_str!("../ecs/schedule_runner.rs");
    assert!(runner_source.contains("internal_systems: &[SceneSystemDescriptor]"));
    assert!(runner_source.contains("native_steps: &[ScheduledSceneStep]"));
    assert!(!runner_source.contains("SceneRuntimeHook"));
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
    assert!(!step_source.contains("ScheduledSceneStepRef::Hook"));
    assert!(step_source.contains(".then(left.id().cmp(right.id()))"));
    assert!(step_source.contains(".then(left.step_rank().cmp(&right.step_rank()))"));
    assert!(!step_source.contains("pub(crate) fn sorted_for_stage("));
    assert!(!step_source.contains("fn to_owned_step("));
    assert!(!step_source.contains("Internal(SceneSystemDescriptor)"));
    assert!(!step_source.contains("Hook(SceneRuntimeHookRegistration)"));
    assert!(step_source.contains("internal_systems: &[SceneSystemDescriptor]"));
    assert!(step_source.contains("native_steps: &'a [Self]"));
    assert!(!step_source.contains("hooks: &[SceneRuntimeHookRegistration]"));
    assert!(
        step_source.contains("let mut next = match self.internal_systems.get(self.internal_index)")
    );
    assert!(step_source.contains("fn should_replace_next_step("));
    assert!(step_source.contains("match current"));
    assert!(!step_source.contains(".map(ScheduledSceneStepRef::Internal)"));
    assert!(!step_source.contains(".is_none_or(|current| compare_step_refs("));
    assert!(step_source.contains("debug_assert!(internal_systems_match_stage("));
    assert!(step_source.contains("debug_assert!(native_steps_match_stage("));
    assert!(step_source.contains("fn internal_systems_match_stage("));
    assert!(step_source.contains("for system in internal_systems"));
    assert!(step_source.contains("if system.stage != stage"));
    assert!(step_source.contains("fn native_steps_match_stage("));
    assert!(step_source.contains("for step in native_steps"));
    assert!(step_source.contains("if step.stage() != stage"));
    assert!(!step_source.contains("internal_systems.iter().all(|system| system.stage == stage)"));
    assert!(!step_source.contains("native_steps.iter().all(|step| step.stage() == stage)"));
    assert!(!step_source.contains("hooks.iter().all(|hook| hook.descriptor().stage == stage)"));
    assert!(!step_source.contains("internal_systems: Vec<SceneSystemDescriptor>"));
    assert!(!step_source.contains("native_steps: Vec<Self>"));
    assert!(!step_source.contains("hooks: Vec<SceneRuntimeHookRegistration>"));
    assert!(!step_source.contains("filter(|system| system.stage == stage)"));
    assert!(!step_source.contains("filter(|hook| hook.descriptor().stage == stage)"));

    let schedule_source = include_str!("../ecs/schedule_stage_plan.rs");
    assert!(schedule_source.contains("struct SceneScheduleStagePlan"));
    assert!(schedule_source.contains("from_registry("));
    assert!(!schedule_source.contains("from_schedule("));
    assert!(
        schedule_source.contains(
            "internal_systems_by_stage: [Vec<SceneSystemDescriptor>; SystemStage::COUNT]"
        )
    );
    assert!(
        schedule_source
            .contains("native_steps_by_stage: [Vec<ScheduledSceneStep>; SystemStage::COUNT]")
    );
    assert!(schedule_source.contains("let systems = registry.systems();"));
    assert!(!schedule_source.contains("let runtime_systems: Vec<_>"));
    assert!(!schedule_source.contains("registry.runtime_systems().iter().collect()"));
    assert!(!schedule_source.contains("same_stage_by_id"));
    assert!(schedule_source.contains("let all_nodes = PlanNodes::new(registry);"));
    assert!(schedule_source.contains("registry: &'a SceneSystemRegistry"));
    assert!(
        schedule_source
            .contains("runtime_systems: impl Iterator<Item = &'registry BoxedRuntimeSceneSystem>")
    );
    assert!(schedule_source.contains("let mut stage_order = Vec::with_capacity(stages.len());"));
    assert!(schedule_source.contains("for stage in stages.iter().copied()"));
    assert!(schedule_source.contains("stage_order.push(stage);"));
    assert!(schedule_source.contains("stages: stage_order,"));
    assert!(!schedule_source.contains("stages: stages.to_vec(),"));
    assert!(
        schedule_source
            .contains("let internal_system_counts = internal_system_counts_by_stage(systems);")
    );
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
    assert!(
        schedule_owner_source
            .contains("fn refresh_or_defer_executor_plan(&mut self) -> Result<(), ScheduleError>")
    );
    assert!(schedule_owner_source.contains("fn refresh_executor_plan(&mut self)"));
    assert!(schedule_owner_source.contains("self.refresh_executor_plan()?;"));
    assert!(schedule_owner_source.contains("self.executor_plan_dirty = true;"));
    assert!(schedule_owner_source.contains("taken_native_system_ids: Vec<String>"));
    assert!(schedule_owner_source.contains("taken_runtime_system_count: usize"));
    assert!(
        schedule_owner_source
            .contains("self.taken_native_system_ids.push(system.id().to_string());")
    );
    assert!(schedule_owner_source.contains("self.taken_runtime_system_count += 1;"));
    assert!(schedule_owner_source.contains("let system_id = system.id();"));
    assert!(
        schedule_owner_source
            .contains("remove_taken_system_id(&mut self.taken_native_system_ids, system_id);")
    );
    assert!(schedule_owner_source.contains("self.taken_runtime_system_count -= 1;"));
    assert!(schedule_owner_source.contains("taken_native_system_ids: Vec::new()"));
    assert!(schedule_owner_source.contains("taken_runtime_system_count: 0"));
    assert!(schedule_owner_source.contains("taken_system_id_exists("));
    assert!(
        schedule_owner_source
            .contains("fn taken_system_id_exists(taken_system_ids: &[String], id: &str)")
    );
    assert!(!schedule_owner_source.contains("fn remove_taken_system_id("));
    assert!(schedule_owner_source.contains("for taken_id in taken_system_ids"));
    assert!(schedule_owner_source.contains("if taken_id.as_str() == id"));
    assert!(schedule_owner_source.contains("let mut index = 0_usize;"));
    assert!(schedule_owner_source.contains("while index < taken_system_ids.len()"));
    assert!(schedule_owner_source.contains("taken_system_ids[index].as_str() == id"));
    assert!(schedule_owner_source.contains("index += 1;"));
    assert!(schedule_owner_source.contains("taken_system_ids.swap_remove(index);"));
    assert!(!schedule_owner_source.contains(".any(|taken_id| taken_id.as_str() == id)"));
    assert!(!schedule_owner_source.contains(".position(|taken_id| taken_id.as_str() == id)"));
    assert!(schedule_owner_source.contains("impl<'de> Deserialize<'de> for Schedule"));
    assert!(schedule_owner_source.contains("impl Clone for Schedule"));
    let default_stage_order_body = schedule_owner_source
        .split("pub fn default_stage_order() -> Vec<SystemStage>")
        .nth(1)
        .and_then(|text| text.split("fn default_system_registry").next())
        .expect("read default_stage_order body");
    assert!(default_stage_order_body.contains("Vec::with_capacity(SystemStage::ORDER.len())"));
    assert!(default_stage_order_body.contains("for stage in SystemStage::ORDER.iter().copied()"));
    assert!(default_stage_order_body.contains("stages.push(stage);"));
    assert!(default_stage_order_body.contains("stages"));
    assert!(!default_stage_order_body.contains("SystemStage::ORDER.to_vec()"));
    assert!(!schedule_owner_source.contains("pub stages: Vec<SystemStage>"));
    assert!(!schedule_owner_source.contains("use std::collections::BTreeSet;"));
    assert!(!schedule_owner_source.contains("taken_native_system_ids: BTreeSet<String>"));
    assert!(!schedule_owner_source.contains("BTreeSet::new()"));
    assert!(!schedule_owner_source.contains("self.taken_native_system_ids.insert("));
    assert!(!schedule_owner_source.contains("self.taken_native_system_ids.remove(&system_id)"));
    assert!(
        !schedule_owner_source
            .contains("taken_native_system_ids.retain(|taken_id| taken_id.as_str() != id);")
    );
    assert!(!schedule_owner_source.contains("SceneScheduleStagePlan::from_schedule(self)"));
    let restore_native_system_body = schedule_owner_source
        .split("pub(crate) fn restore_native_system(&mut self, system: BoxedSceneSystem)")
        .nth(1)
        .and_then(|text| text.split("fn from_parts(").next())
        .expect("read Schedule::restore_native_system body");
    assert!(!restore_native_system_body.contains("system.id().to_string()"));
    let borrowed_restore_id_index = restore_native_system_body
        .find("let system_id = system.id();")
        .expect("restore should borrow the native system id");
    let restore_system_index = restore_native_system_body
        .find("self.systems.restore_native_system(system);")
        .expect("restore should move the native system back into the registry");
    assert!(borrowed_restore_id_index < restore_system_index);

    let registry_source = include_str!("../ecs/scene_system_registry.rs");
    assert!(registry_source.contains("native_system_steps_by_stage("));
    assert!(registry_source.contains("let native_step_counts ="));
    assert!(
        registry_source
            .contains("native_step_counts_by_stage(&self.native_systems, self.runtime_systems())")
    );
    assert!(
        registry_source
            .contains("let mut by_stage = native_step_groups_with_capacity(&native_step_counts);")
    );
    assert!(registry_source.contains("fn native_step_counts_by_stage("));
    assert_eq!(
        registry_source
            .matches("runtime_systems: RuntimeSystems<'_>")
            .count(),
        2,
        "step counting and conflict-graph sizing must consume slots without a temporary Vec"
    );
    assert!(!registry_source.contains("let runtime_systems: Vec<_>"));
    assert!(registry_source.contains("let step_count = if system.has_deferred_commands() {"));
    assert!(registry_source.contains("counts[system.stage().rank()] += step_count;"));
    assert_eq!(
        registry_source
            .matches("for system in self.runtime_systems()")
            .count(),
        2,
        "stage-plan and conflict-graph construction must iterate stable slots directly"
    );
    assert!(registry_source.contains("fn native_step_groups_with_capacity("));
    assert!(registry_source.contains("Vec::with_capacity(native_step_counts[stage_index])"));
    assert!(!registry_source.contains("fn empty_native_step_groups("));
    assert!(!registry_source.contains("std::array::from_fn(|_| Vec::new())"));
    assert!(registry_source.contains("insert_system_sorted(&mut self.systems, descriptor);"));
    assert!(registry_source.contains("runtime_systems: RuntimeSystemSlots"));
    assert!(registry_source.contains("self.runtime_systems.take(id)"));
    assert!(registry_source.contains("self.runtime_systems.restore(system)"));
    let runtime_slots_source = include_str!("../ecs/scene_system_registry/runtime_system_slots.rs");
    assert!(runtime_slots_source.contains("slots: Vec<RuntimeSystemSlot>"));
    assert!(runtime_slots_source.contains("indices: HashMap<String, usize>"));
    let runtime_take_body = runtime_slots_source
        .split("pub(super) fn take(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn remove(").next())
        .expect("read RuntimeSystemSlots::take body");
    assert!(runtime_take_body.contains("self.indices.get(id)"));
    assert!(runtime_take_body.contains("self.slots.get_mut(index)?.system.take()"));
    assert!(!runtime_take_body.contains("self.slots.remove("));
    let runtime_restore_body = runtime_slots_source
        .split("pub(super) fn restore(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn insert(").next())
        .expect("read RuntimeSystemSlots::restore body");
    assert!(runtime_restore_body.contains("self.indices"));
    assert!(runtime_restore_body.contains("slot.system = Some(system)"));
    assert!(!runtime_slots_source.contains("insert_runtime_system_sorted("));
    assert_eq!(
        registry_source
            .matches("insert_native_system_sorted(&mut self.native_systems, system);")
            .count(),
        4,
        "native registration variants and restore must share ordered insertion"
    );
    assert!(registry_source.contains("fn insert_system_sorted("));
    assert!(registry_source.contains("fn compare_system_descriptors("));
    assert!(registry_source.contains("fn insert_native_system_sorted("));
    assert!(registry_source.contains("fn compare_native_systems("));
    assert!(registry_source.contains(".binary_search_by(|existing|"));
    assert!(registry_source.contains("Ok(index) | Err(index) => index"));
    assert!(registry_source.contains("systems.insert(insert_index, descriptor);"));
    assert!(registry_source.contains("systems.insert(insert_index, system);"));
    assert!(!registry_source.contains(".unwrap_or_else(|index| index)"));
    assert!(
        registry_source.contains("let system = match system.into_scene_system(metadata, world)")
    );
    assert!(registry_source.contains("Err(source) => {"));
    assert!(registry_source.contains("return Err(ScheduleError::SystemParam"));
    assert!(!registry_source.contains(".map_err(|source| ScheduleError::SystemParam"));
    assert!(registry_source.contains("SystemsForStage::new(&self.systems, stage)"));
    assert!(registry_source.contains("struct SystemsForStage<'registry>"));
    assert!(registry_source.contains("impl<'registry> Iterator for SystemsForStage<'registry>"));
    assert!(registry_source.contains("return Some(system);"));
    assert!(!registry_source.contains(".filter(move |system| system.stage == stage)"));
    assert!(registry_source.contains("NativeSystemsForStage::new(&self.native_systems, stage)"));
    assert!(registry_source.contains("struct NativeSystemsForStage<'registry>"));
    assert!(
        registry_source.contains("impl<'registry> Iterator for NativeSystemsForStage<'registry>")
    );
    assert!(registry_source.contains("for system in self.systems.by_ref()"));
    assert!(registry_source.contains("return Some(system.as_ref());"));
    assert!(!registry_source.contains(".map(|system| system.as_ref())"));
    assert!(registry_source.contains("registered_system_id_exists(&self.systems, id)"));
    assert!(
        registry_source.contains("registered_native_system_id_exists(&self.native_systems, id)")
    );
    assert!(registry_source.contains("fn registered_system_id_exists("));
    assert!(registry_source.contains("fn registered_native_system_id_exists("));
    assert!(registry_source.contains("for system in systems"));
    assert!(registry_source.contains("return true;"));
    assert!(!registry_source.contains(".iter().any(|system| system.id == id)"));
    assert!(!registry_source.contains(".iter().any(|system| system.id() == id)"));
    let take_native_system_body = registry_source
        .split("pub(crate) fn take_native_system(&mut self, id: &str)")
        .nth(1)
        .and_then(|text| text.split("pub(crate) fn restore_native_system").next())
        .expect("read SceneSystemRegistry::take_native_system body");
    assert!(take_native_system_body.contains("let mut index = 0_usize;"));
    assert!(take_native_system_body.contains("while index < self.native_systems.len()"));
    assert!(take_native_system_body.contains("self.native_systems[index].id() == id"));
    assert!(take_native_system_body.contains("return Some(self.native_systems.remove(index));"));
    assert!(take_native_system_body.contains("index += 1;"));
    assert!(take_native_system_body.contains("None"));
    assert!(!take_native_system_body.contains(".position(|system| system.id() == id)"));
    assert!(!registry_source.contains("sort_systems(&mut self.systems);"));
    assert!(!registry_source.contains("sort_native_systems(&mut self.native_systems);"));
    assert!(!registry_source.contains("fn sort_systems("));
    assert!(!registry_source.contains("fn sort_native_systems("));

    assert!(registry_source.contains("native_system_conflict_graph_for_stage("));
    assert!(
        registry_source.contains("let node_count = native_conflict_graph_node_count_for_stage(")
    );
    assert!(!registry_source.contains(".runtime_systems().iter().collect()"));
    assert!(registry_source.contains("let mut nodes = Vec::with_capacity(node_count);"));
    assert!(registry_source.contains("nodes.push(ScheduleConflictNode::new("));
    assert!(registry_source.contains("nodes.push(ScheduleConflictNode::barrier("));
    assert!(registry_source.contains("ScheduleConflictGraph::from_node_vec(nodes)"));
    assert!(registry_source.contains("fn native_conflict_graph_node_count_for_stage("));
    assert!(
        registry_source.contains("count += if system.has_deferred_commands() { 2 } else { 1 };")
    );
    assert!(registry_source.contains("if system.stage() == stage {\n            count += 1;"));
    let native_conflict_graph_body = registry_source
        .split("pub fn native_system_conflict_graph_for_stage(")
        .nth(1)
        .and_then(|text| text.split("pub(crate) fn take_native_system").next())
        .expect("read native system conflict-graph builder");
    assert!(!native_conflict_graph_body.contains(".filter(|system| system.stage() == stage)"));
    assert!(!native_conflict_graph_body.contains(".flat_map(|system|"));

    let graph_source = include_str!("../ecs/schedule_conflict_graph.rs");
    assert!(
        graph_source
            .contains("pub(crate) fn from_node_vec(nodes: Vec<ScheduleConflictNode>) -> Self")
    );
    assert!(graph_source.contains("Self::from_node_vec(nodes)"));
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
