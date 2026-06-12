use crate::core::CoreError;
use crate::plugin::{
    RuntimeExtensionRegistry, SceneRuntimeHook, SceneRuntimeHookContext,
    SceneRuntimeHookDescriptor, SceneRuntimeHookRegistration,
};
use crate::scene::SystemStage;
use std::time::Duration;

#[test]
fn runtime_extension_registry_rejects_non_namespaced_scene_hook_ids() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_scene_hook(scene_hook_registration(
            "weather",
            "weather",
            SystemStage::Update,
        ))
        .unwrap_err();

    assert!(error.to_string().contains("dot-separated namespace"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_scene_hook(scene_hook_registration(
            "weather.scene.PostUpdate",
            "weather",
            SystemStage::Update,
        ))
        .unwrap_err();

    assert!(error.to_string().contains("lowercase ASCII"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_scene_hook(scene_hook_registration(
            "weather.scene.post/update",
            "weather",
            SystemStage::Update,
        ))
        .unwrap_err();

    assert!(error.to_string().contains("lowercase ASCII"));
}

#[test]
fn runtime_extension_registry_rejects_invalid_scene_hook_plugin_ids() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_scene_hook(scene_hook_registration(
            "weather.scene.post_update",
            "Weather",
            SystemStage::Update,
        ))
        .unwrap_err();

    assert!(error.to_string().contains("plugin_id"));
    assert!(error.to_string().contains("lowercase ASCII"));
}

#[test]
fn runtime_extension_registry_collects_scene_hook_contributions_in_stage_order() {
    let mut registry = RuntimeExtensionRegistry::default();

    registry
        .register_scene_hook(ordered_scene_hook_registration(
            "weather.scene.update-late",
            SystemStage::Update,
            20,
        ))
        .expect("late update hook contribution");
    registry
        .register_scene_hook(ordered_scene_hook_registration(
            "weather.scene.fixed",
            SystemStage::FixedUpdate,
            0,
        ))
        .expect("fixed hook contribution");
    registry
        .register_scene_hook(ordered_scene_hook_registration(
            "weather.scene.update-early",
            SystemStage::Update,
            -10,
        ))
        .expect("early update hook contribution");

    let hook_ids = registry
        .scene_hooks()
        .iter()
        .map(|hook| hook.descriptor().id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        hook_ids,
        vec![
            "weather.scene.fixed",
            "weather.scene.update-early",
            "weather.scene.update-late",
        ]
    );
}

#[test]
fn runtime_extension_registry_rejects_duplicate_and_invalid_scene_hooks() {
    let mut registry = RuntimeExtensionRegistry::default();

    registry
        .register_scene_hook(ordered_scene_hook_registration(
            "weather.scene.update",
            SystemStage::Update,
            0,
        ))
        .expect("first hook contribution");
    let duplicate = registry
        .register_scene_hook(ordered_scene_hook_registration(
            "weather.scene.update",
            SystemStage::Update,
            1,
        ))
        .unwrap_err();
    assert!(duplicate
        .to_string()
        .contains("scene hook weather.scene.update already registered"));

    let invalid = registry
        .register_scene_hook(SceneRuntimeHookRegistration::new(
            SceneRuntimeHookDescriptor::new("cloud.scene.update", "weather", SystemStage::Update),
            Noop,
        ))
        .unwrap_err();
    assert!(invalid
        .to_string()
        .contains("scene hook cloud.scene.update must be prefixed by plugin id weather"));
}

#[test]
fn level_tick_dispatches_installed_scene_hooks_in_schedule_order() {
    let runtime = crate::core::CoreRuntime::new();
    runtime
        .register_module(crate::scene::module_descriptor())
        .unwrap();
    runtime
        .activate_module(crate::scene::SCENE_MODULE_NAME)
        .unwrap();

    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_scene_hook(recording_scene_hook_registration(
            "weather.scene.update",
            SystemStage::Update,
            0,
            "update",
        ))
        .expect("update hook contribution");
    registry
        .register_scene_hook(recording_scene_hook_registration(
            "weather.scene.pre-update",
            SystemStage::PreUpdate,
            0,
            "pre-update",
        ))
        .expect("pre-update hook contribution");
    runtime
        .install_scene_runtime_hooks(&registry)
        .expect("install scene hooks into core runtime");

    let level = crate::scene::create_default_level(&runtime.handle()).unwrap();
    let advance = runtime.advance_time_by(Duration::from_secs_f32(1.0 / 60.0), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        level.registered_subsystems(),
        vec!["pre-update".to_string(), "update".to_string()]
    );
}

#[test]
fn level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps() {
    let runtime = crate::core::CoreRuntime::new();
    runtime
        .register_module(crate::scene::module_descriptor())
        .unwrap();
    runtime
        .activate_module(crate::scene::SCENE_MODULE_NAME)
        .unwrap();
    runtime.set_fixed_timestep(Duration::from_millis(10));

    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_scene_hook(recording_scene_hook_registration(
            "weather.scene.fixed-first",
            SystemStage::FixedFirst,
            0,
            "fixed-first",
        ))
        .expect("fixed first hook contribution");
    registry
        .register_scene_hook(recording_scene_hook_registration(
            "weather.scene.fixed-update",
            SystemStage::FixedUpdate,
            0,
            "fixed-update",
        ))
        .expect("fixed update hook contribution");
    registry
        .register_scene_hook(recording_scene_hook_registration(
            "weather.scene.fixed-post-update",
            SystemStage::FixedPostUpdate,
            0,
            "fixed-post-update",
        ))
        .expect("fixed post-update hook contribution");
    runtime
        .install_scene_runtime_hooks(&registry)
        .expect("install fixed scene hooks into core runtime");

    let level = crate::scene::create_default_level(&runtime.handle()).unwrap();
    let advance = runtime.advance_time_by(Duration::from_millis(25), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        level.registered_subsystems(),
        vec![
            "fixed-first".to_string(),
            "fixed-update".to_string(),
            "fixed-post-update".to_string(),
            "fixed-first".to_string(),
            "fixed-update".to_string(),
            "fixed-post-update".to_string(),
        ]
    );
}

#[test]
fn level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained() {
    let runtime = crate::core::CoreRuntime::new();
    runtime
        .register_module(crate::scene::module_descriptor())
        .unwrap();
    runtime
        .activate_module(crate::scene::SCENE_MODULE_NAME)
        .unwrap();
    runtime.set_fixed_timestep(Duration::from_millis(10));

    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_scene_hook(recording_scene_hook_registration(
            "weather.scene.fixed-update",
            SystemStage::FixedUpdate,
            0,
            "fixed-update",
        ))
        .expect("fixed update hook contribution");
    runtime
        .install_scene_runtime_hooks(&registry)
        .expect("install fixed scene hooks into core runtime");

    let level = crate::scene::create_default_level(&runtime.handle()).unwrap();
    let advance = runtime.advance_time_by(Duration::from_millis(5), 8);
    assert_eq!(advance.fixed_step_plan().step_count, 0);

    level.tick(&runtime.handle(), advance).unwrap();

    assert!(level.registered_subsystems().is_empty());
}

#[test]
fn level_tick_fixed_loop_steps_are_capped_by_runtime_time_advance() {
    let runtime = crate::core::CoreRuntime::new();
    runtime
        .register_module(crate::scene::module_descriptor())
        .unwrap();
    runtime
        .activate_module(crate::scene::SCENE_MODULE_NAME)
        .unwrap();
    runtime.set_fixed_timestep(Duration::from_millis(10));

    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_scene_hook(recording_scene_hook_registration(
            "weather.scene.fixed-update",
            SystemStage::FixedUpdate,
            0,
            "fixed-update",
        ))
        .expect("fixed update hook contribution");
    runtime
        .install_scene_runtime_hooks(&registry)
        .expect("install fixed scene hooks into core runtime");

    let level = crate::scene::create_default_level(&runtime.handle()).unwrap();
    let advance = runtime.advance_time_by(Duration::from_millis(50), 3);
    assert_eq!(advance.fixed_step_plan().step_count, 3);
    assert_eq!(
        advance.fixed_step_plan().remaining_overstep,
        Duration::from_millis(20)
    );

    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        level.registered_subsystems(),
        vec![
            "fixed-update".to_string(),
            "fixed-update".to_string(),
            "fixed-update".to_string(),
        ]
    );
}

fn scene_hook_registration(
    id: impl Into<String>,
    plugin_id: impl Into<String>,
    stage: SystemStage,
) -> SceneRuntimeHookRegistration {
    SceneRuntimeHookRegistration::new(SceneRuntimeHookDescriptor::new(id, plugin_id, stage), Noop)
}

fn ordered_scene_hook_registration(
    id: &str,
    stage: SystemStage,
    order: i32,
) -> SceneRuntimeHookRegistration {
    SceneRuntimeHookRegistration::new(
        SceneRuntimeHookDescriptor::new(id, "weather", stage).with_order(order),
        Noop,
    )
}

fn recording_scene_hook_registration(
    id: &str,
    stage: SystemStage,
    order: i32,
    label: &'static str,
) -> SceneRuntimeHookRegistration {
    SceneRuntimeHookRegistration::new(
        SceneRuntimeHookDescriptor::new(id, "weather", stage).with_order(order),
        Recording { label },
    )
}

#[derive(Clone, Debug)]
struct Noop;

impl SceneRuntimeHook for Noop {
    fn run(&self, _context: SceneRuntimeHookContext<'_>) -> Result<(), CoreError> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Recording {
    label: &'static str,
}

impl SceneRuntimeHook for Recording {
    fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), CoreError> {
        context.level.register_subsystem(self.label);
        Ok(())
    }
}
