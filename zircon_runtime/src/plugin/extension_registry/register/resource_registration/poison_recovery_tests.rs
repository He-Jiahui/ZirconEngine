use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use crate::scene::ecs::Resource;
use crate::scene::World;

use super::ResourceRegistration;

#[derive(Debug, PartialEq, Eq)]
struct RecoveredResource(usize);

impl Resource for RecoveredResource {}

#[test]
fn resource_registration_factory_can_retry_after_a_panicking_invocation() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let initializer_attempts = Arc::clone(&attempts);
    let registration = ResourceRegistration::new(move || {
        let attempt = initializer_attempts.fetch_add(1, Ordering::SeqCst);
        assert!(attempt > 0, "first resource initialization attempt fails");
        RecoveredResource(attempt)
    });
    let mut world = World::new();

    let first_attempt = registration.apply(&mut world);

    assert!(first_attempt.is_err());
    assert!(!world.contains_resource::<RecoveredResource>());

    registration
        .apply(&mut world)
        .expect("second resource initialization attempt should succeed");

    assert_eq!(
        world.get_resource::<RecoveredResource>(),
        Some(&RecoveredResource(1))
    );
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[test]
fn resource_registration_factory_builds_a_fresh_value_for_each_world() {
    let factory_calls = Arc::new(AtomicUsize::new(0));
    let calls = Arc::clone(&factory_calls);
    let registration =
        ResourceRegistration::new(move || RecoveredResource(calls.fetch_add(1, Ordering::SeqCst)));
    let mut first_world = World::new();
    let mut second_world = World::new();

    registration
        .apply(&mut first_world)
        .expect("first world resource initialization should succeed");
    registration
        .apply(&mut second_world)
        .expect("second world resource initialization should succeed");

    assert_eq!(
        first_world.get_resource::<RecoveredResource>(),
        Some(&RecoveredResource(0))
    );
    assert_eq!(
        second_world.get_resource::<RecoveredResource>(),
        Some(&RecoveredResource(1))
    );
    assert_eq!(factory_calls.load(Ordering::SeqCst), 2);
}

#[test]
fn resource_registration_factory_allows_concurrent_world_initialization() {
    let active_calls = Arc::new(AtomicUsize::new(0));
    let peak_active_calls = Arc::new(AtomicUsize::new(0));
    let registration = Arc::new(ResourceRegistration::new({
        let active_calls = Arc::clone(&active_calls);
        let peak_active_calls = Arc::clone(&peak_active_calls);
        move || {
            let active = active_calls.fetch_add(1, Ordering::SeqCst) + 1;
            peak_active_calls.fetch_max(active, Ordering::SeqCst);
            let deadline = Instant::now() + Duration::from_millis(250);
            while active_calls.load(Ordering::SeqCst) < 2 && Instant::now() < deadline {
                thread::yield_now();
            }
            thread::sleep(Duration::from_millis(20));
            active_calls.fetch_sub(1, Ordering::SeqCst);
            RecoveredResource(active)
        }
    }));
    let start = Arc::new(Barrier::new(2));

    thread::scope(|scope| {
        for _ in 0..2 {
            let registration = Arc::clone(&registration);
            let start = Arc::clone(&start);
            scope.spawn(move || {
                start.wait();
                let mut world = World::new();
                registration
                    .apply(&mut world)
                    .expect("concurrent resource initialization should succeed");
            });
        }
    });

    assert_eq!(peak_active_calls.load(Ordering::SeqCst), 2);
    assert_eq!(active_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn resource_registration_duplicate_registration_returns_typed_error() {
    let mut registry = super::RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("resource_registration.runtime")
        .expect("resource owner should be interned before registration");

    registry
        .register_resource(owner, || RecoveredResource(1))
        .expect("first resource registration should succeed");
    let error = registry
        .register_resource(owner, || RecoveredResource(2))
        .expect_err("duplicate resource registration should return an error");

    assert!(matches!(
        error,
        crate::plugin::RuntimeExtensionRegistryError::DuplicatePluginResource(type_name)
            if type_name == std::any::type_name::<RecoveredResource>()
    ));
}

#[test]
fn resource_registration_rejects_unknown_owner_before_mutating_registry() {
    let mut registry = super::RuntimeExtensionRegistry::default();
    let error = registry
        .register_resource(crate::plugin::PluginModuleId::from_raw(4), || {
            RecoveredResource(1)
        })
        .expect_err("resource registration must require an interned owner");

    assert!(matches!(
        error,
        crate::plugin::RuntimeExtensionRegistryError::InvalidPluginModule(message)
            if message == "unknown plugin module owner 4"
    ));
    assert_eq!(registry.plugin_resources().count(), 0);
}

#[test]
fn resource_registration_rejects_non_runtime_owner_before_mutating_registry() {
    let mut registry = super::RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("resource_registration.editor")
        .expect("editor owner should be interned for the boundary test");

    let error = registry
        .register_resource(owner, || RecoveredResource(1))
        .expect_err("resource registration must require a runtime owner");

    assert!(matches!(
        error,
        crate::plugin::RuntimeExtensionRegistryError::InvalidPluginModule(message)
            if message == "resource owner `resource_registration.editor` must use the <plugin>.runtime module form"
    ));
    assert_eq!(registry.plugin_resources().count(), 0);
}

#[test]
fn resource_factory_panic_is_reported_without_partial_world_mutation() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let mut registry = super::RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("resource_registration.runtime")
        .expect("resource owner should be interned before registration");
    let factory_attempts = Arc::clone(&attempts);
    registry
        .register_resource(owner, move || {
            if factory_attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                panic!("resource factory failed");
            }
            RecoveredResource(2)
        })
        .expect("resource registration should succeed");

    let mut first_world = World::new();
    let first_error = registry
        .apply_to_world(&mut first_world)
        .expect_err("resource factory panic should become a world registration error");
    assert!(matches!(
        first_error,
        crate::plugin::RuntimeExtensionRegistryError::WorldRegistration(message)
            if message.contains("resource:")
                && message.contains(std::any::type_name::<RecoveredResource>())
                && message.contains("resource factory failed")
    ));
    assert!(!first_world.contains_resource::<RecoveredResource>());

    let mut second_world = World::new();
    registry
        .apply_to_world(&mut second_world)
        .expect("a failed resource factory must remain retryable for another world");
    assert_eq!(
        second_world.get_resource::<RecoveredResource>(),
        Some(&RecoveredResource(2))
    );
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}
