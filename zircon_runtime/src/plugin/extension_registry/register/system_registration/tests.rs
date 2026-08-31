use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use super::*;

#[test]
fn typed_scene_system_callback_state_is_private_per_world() {
    let observed = Arc::new(Mutex::new(Vec::new()));
    let factory_builds = Arc::new(AtomicUsize::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.typed").unwrap();
    let observed_for_factory = Arc::clone(&observed);
    let factory_builds_for_factory = Arc::clone(&factory_builds);

    registry
        .register_native_system::<(), _>(
            owner,
            "tests.typed.private-state",
            SystemStage::Update,
            move || {
                factory_builds_for_factory.fetch_add(1, Ordering::SeqCst);
                let observed = Arc::clone(&observed_for_factory);
                let mut calls = 0usize;
                move |_| {
                    calls += 1;
                    observed.lock().unwrap().push(calls);
                }
            },
        )
        .register()
        .unwrap();

    let registration = registry.plugin_systems().next().unwrap().1;
    let mut first_world = World::empty();
    let mut second_world = World::empty();
    let mut first = registration.build(&mut first_world).unwrap();
    let mut second = registration.build(&mut second_world).unwrap();

    first.run(&mut first_world);
    second.run(&mut second_world);

    assert_eq!(*observed.lock().unwrap(), vec![1, 1]);
    assert_eq!(factory_builds.load(Ordering::SeqCst), 2);
}

#[test]
fn external_scene_system_callback_state_is_private_per_world() {
    let observed = Arc::new(Mutex::new(Vec::new()));
    let factory_builds = Arc::new(AtomicUsize::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.external").unwrap();
    let observed_for_factory = Arc::clone(&observed);
    let factory_builds_for_factory = Arc::clone(&factory_builds);

    registry
        .register_external_native_system(
            owner,
            "tests.external.private-state",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |_world| Ok(SystemParamAccess::default()),
            move || {
                factory_builds_for_factory.fetch_add(1, Ordering::SeqCst);
                let observed = Arc::clone(&observed_for_factory);
                let mut calls = 0usize;
                move || {
                    calls += 1;
                    observed.lock().unwrap().push(calls);
                }
            },
        )
        .register()
        .unwrap();

    let registration = registry.plugin_systems().next().unwrap().1;
    let mut first_world = World::empty();
    let mut second_world = World::empty();
    let mut first = registration.build(&mut first_world).unwrap();
    let mut second = registration.build(&mut second_world).unwrap();

    first.run_without_world();
    second.run_without_world();

    assert_eq!(*observed.lock().unwrap(), vec![1, 1]);
    assert_eq!(factory_builds.load(Ordering::SeqCst), 2);
}

#[test]
fn external_scene_system_callbacks_overlap_across_worlds() {
    #[derive(Default)]
    struct CallbackProgress {
        active: usize,
        max_active: usize,
        both_started: bool,
    }

    let progress = Arc::new((Mutex::new(CallbackProgress::default()), Condvar::new()));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("tests.external.concurrent")
        .unwrap();
    let progress_for_system = Arc::clone(&progress);

    registry
        .register_external_native_system(
            owner,
            "tests.external.concurrent-worlds",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |_world| Ok(SystemParamAccess::default()),
            move || {
                let progress = Arc::clone(&progress_for_system);
                move || {
                    let (progress_lock, progress_changed) = &*progress;
                    let mut progress = progress_lock.lock().unwrap();
                    progress.active += 1;
                    progress.max_active = progress.max_active.max(progress.active);
                    if progress.active == 2 {
                        progress.both_started = true;
                        progress_changed.notify_all();
                    }
                    let (mut progress, _) = progress_changed
                        .wait_timeout_while(progress, Duration::from_secs(1), |progress| {
                            !progress.both_started
                        })
                        .unwrap();
                    progress.active -= 1;
                    progress_changed.notify_all();
                }
            },
        )
        .register()
        .unwrap();

    let registration = registry.plugin_systems().next().unwrap().1;
    let mut first_world = World::empty();
    let mut second_world = World::empty();
    let mut first = registration.build(&mut first_world).unwrap();
    let mut second = registration.build(&mut second_world).unwrap();

    let first = thread::spawn(move || first.run_without_world());
    let second = thread::spawn(move || second.run_without_world());
    first.join().unwrap();
    second.join().unwrap();

    assert_eq!(progress.0.lock().unwrap().max_active, 2);
}
