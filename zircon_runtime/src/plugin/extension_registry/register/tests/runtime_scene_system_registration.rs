use std::time::Duration;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::*;
use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::CoreRuntime;
use crate::scene::ecs::{RuntimeSceneSystemContext, SystemTickContext};
use crate::scene::{create_default_level, module_descriptor};

#[test]
fn runtime_scene_system_callback_state_is_private_per_instance() {
    let observed = Arc::new(Mutex::new(Vec::new()));
    let factory_builds = Arc::new(AtomicUsize::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.runtime").unwrap();
    let observed_for_factory = Arc::clone(&observed);
    let factory_builds_for_factory = Arc::clone(&factory_builds);

    registry
        .register_runtime_scene_system(
            owner,
            "tests.runtime.private-state",
            SystemStage::Update,
            move || {
                factory_builds_for_factory.fetch_add(1, Ordering::SeqCst);
                let observed = Arc::clone(&observed_for_factory);
                let mut calls = 0usize;
                move |_| {
                    calls += 1;
                    observed.lock().unwrap().push(calls);
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();

    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let registration = registry.plugin_runtime_systems().next().unwrap().1;
    let mut first = registration.build();
    let mut second = registration.build();

    first
        .run(RuntimeSceneSystemContext::new(
            &runtime.handle(),
            &level,
            SystemTickContext::new(
                SystemStage::Update,
                level.world_time().virtual_time().clock_domain_stamp(),
                0,
                None,
                Duration::ZERO,
                Duration::ZERO,
                level.world_generation(),
            ),
        ))
        .unwrap();
    second
        .run(RuntimeSceneSystemContext::new(
            &runtime.handle(),
            &level,
            SystemTickContext::new(
                SystemStage::Update,
                level.world_time().virtual_time().clock_domain_stamp(),
                0,
                None,
                Duration::ZERO,
                Duration::ZERO,
                level.world_generation(),
            ),
        ))
        .unwrap();

    assert_eq!(*observed.lock().unwrap(), vec![1, 1]);
    assert_eq!(factory_builds.load(Ordering::SeqCst), 2);
}
