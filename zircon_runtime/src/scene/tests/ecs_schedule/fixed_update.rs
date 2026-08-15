use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::math::Real;
use crate::core::CoreRuntime;
use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::{create_default_level, module_descriptor, SystemStage};

#[test]
fn level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps() {
    let (runtime, level, events) = fixed_update_fixture();
    runtime.set_fixed_timestep(Duration::from_millis(10));
    install_fixed_update_recorder(&level, events.clone());

    let advance = runtime.advance_time_by(Duration::from_millis(35), 8);
    assert_eq!(advance.fixed_step_plan().step_count, 3);

    level.tick(&runtime.handle(), advance).unwrap();

    let events = events.lock().unwrap();
    assert_eq!(events.as_slice(), &[0.01, 0.01, 0.01]);
}

#[test]
fn fixed_loop_stages_run_per_fixed_step() {
    let (runtime, level, _) = fixed_update_fixture();
    runtime.set_fixed_timestep(Duration::from_millis(10));
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("plugins01.runtime").unwrap();
    for (stage, label) in [
        (SystemStage::FixedFirst, "fixed-first"),
        (SystemStage::FixedUpdate, "fixed-update"),
        (SystemStage::FixedPostUpdate, "fixed-post-update"),
    ] {
        let events_for_system = Arc::clone(&events);
        registry
            .register_runtime_scene_system(
                owner,
                format!("plugins01.fixed-loop.{label}"),
                stage,
                move || {
                    let events = Arc::clone(&events_for_system);
                    move |_| {
                        events.lock().unwrap().push(label);
                        Ok(())
                    }
                },
            )
            .register()
            .unwrap();
    }
    apply_runtime_scene_systems(&level, &registry);

    let advance = runtime.advance_time_by(Duration::from_millis(25), 8);
    assert_eq!(advance.fixed_step_plan().step_count, 2);

    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "fixed-first",
            "fixed-update",
            "fixed-post-update",
            "fixed-first",
            "fixed-update",
            "fixed-post-update",
        ]
    );
}

#[test]
fn level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained() {
    let (runtime, level, events) = fixed_update_fixture();
    runtime.set_fixed_timestep(Duration::from_millis(10));
    install_fixed_update_recorder(&level, events.clone());

    let advance = runtime.advance_time_by(Duration::from_millis(5), 8);
    assert_eq!(advance.fixed_step_plan().step_count, 0);

    level.tick(&runtime.handle(), advance).unwrap();

    assert!(events.lock().unwrap().is_empty());
}

#[test]
fn fixed_loop_clamps_to_max_steps_per_frame() {
    let (runtime, level, events) = fixed_update_fixture();
    runtime.set_fixed_timestep(Duration::from_millis(10));
    install_fixed_update_recorder(&level, events.clone());

    let advance = runtime.advance_time_by(Duration::from_millis(55), 4);
    assert_eq!(advance.fixed_step_plan().step_count, 4);

    level.tick(&runtime.handle(), advance).unwrap();

    let events = events.lock().unwrap();
    assert_eq!(events.len(), 4);
    assert!(events.iter().all(|delta| (*delta - 0.01).abs() < 0.000_001));
}

fn fixed_update_fixture() -> (
    CoreRuntime,
    crate::scene::LevelSystem,
    Arc<Mutex<Vec<Real>>>,
) {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    (runtime, level, Arc::new(Mutex::new(Vec::new())))
}

fn install_fixed_update_recorder(level: &crate::scene::LevelSystem, events: Arc<Mutex<Vec<Real>>>) {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("runtime03.runtime").unwrap();
    registry
        .register_runtime_scene_system(
            owner,
            "runtime03.fixed-update.recorder",
            SystemStage::FixedUpdate,
            move || {
                let events = Arc::clone(&events);
                move |context| {
                    events.lock().unwrap().push(context.delta_seconds);
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    apply_runtime_scene_systems(level, &registry);
}

fn apply_runtime_scene_systems(
    level: &crate::scene::LevelSystem,
    registry: &RuntimeExtensionRegistry,
) {
    let plan = registry.world_runtime_extension_plan().unwrap();
    level
        .with_world_mut(|world| plan.apply_to_world(world))
        .unwrap();
}
