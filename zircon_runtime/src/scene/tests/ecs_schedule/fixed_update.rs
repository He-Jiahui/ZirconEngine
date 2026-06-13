use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::math::Real;
use crate::core::CoreRuntime;
use crate::plugin::{
    RuntimeExtensionRegistry, SceneRuntimeHook, SceneRuntimeHookContext,
    SceneRuntimeHookDescriptor, SceneRuntimeHookRegistration,
};
use crate::scene::{create_default_level, module_descriptor, SystemStage, SCENE_MODULE_NAME};

#[test]
fn level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps() {
    let (runtime, level, events) = fixed_update_fixture();
    runtime.set_fixed_timestep(Duration::from_millis(10));
    install_fixed_update_recorder(&runtime, events.clone());

    let advance = runtime.advance_time_by(Duration::from_millis(35), 8);
    assert_eq!(advance.fixed_step_plan().step_count, 3);

    level.tick(&runtime.handle(), advance).unwrap();

    let events = events.lock().unwrap();
    assert_eq!(events.as_slice(), &[0.01, 0.01, 0.01]);
}

#[test]
fn level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained() {
    let (runtime, level, events) = fixed_update_fixture();
    runtime.set_fixed_timestep(Duration::from_millis(10));
    install_fixed_update_recorder(&runtime, events.clone());

    let advance = runtime.advance_time_by(Duration::from_millis(5), 8);
    assert_eq!(advance.fixed_step_plan().step_count, 0);

    level.tick(&runtime.handle(), advance).unwrap();

    assert!(events.lock().unwrap().is_empty());
}

#[test]
fn level_tick_fixed_loop_steps_are_capped_by_runtime_time_advance() {
    let (runtime, level, events) = fixed_update_fixture();
    runtime.set_fixed_timestep(Duration::from_millis(10));
    install_fixed_update_recorder(&runtime, events.clone());

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

fn install_fixed_update_recorder(runtime: &CoreRuntime, events: Arc<Mutex<Vec<Real>>>) {
    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_scene_hook(SceneRuntimeHookRegistration::new(
            SceneRuntimeHookDescriptor::new(
                "runtime03.fixed-update.recorder",
                "runtime03",
                SystemStage::FixedUpdate,
            ),
            FixedUpdateRecorder { events },
        ))
        .unwrap();
    runtime.install_scene_runtime_hooks(&registry).unwrap();
}

#[derive(Debug)]
struct FixedUpdateRecorder {
    events: Arc<Mutex<Vec<Real>>>,
}

impl SceneRuntimeHook for FixedUpdateRecorder {
    fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), crate::core::CoreError> {
        self.events.lock().unwrap().push(context.delta_seconds);
        Ok(())
    }
}
