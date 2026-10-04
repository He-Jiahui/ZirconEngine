use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use super::*;
use crate::scene::{WorldRuntimeExtensionError, WorldRuntimeExtensionRegistration};

#[test]
fn world_runtime_extension_callbacks_apply_from_a_short_lock_snapshot() {
    let source = include_str!("../world_driver.rs");
    let normalized = source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let snapshot = ["fnruntime_extension_plan_", "snapshot"].concat();
    let direct_apply = [
        "lock_poison_recovered(&self.runtime_extensions)",
        ".apply_to_world",
    ]
    .concat();

    assert!(normalized.contains(&snapshot));
    assert!(!normalized.contains(&direct_apply));
}

#[test]
fn world_runtime_extension_snapshot_survives_a_later_plan_publication() {
    let driver = WorldDriver::default();
    driver
        .install_world_runtime_extension_plan(extension_plan("before"))
        .expect("initial extension plan");
    let before = driver.runtime_extension_plan_snapshot();

    driver
        .install_world_runtime_extension_plan(extension_plan("after"))
        .expect("replacement extension plan");
    let after = driver.runtime_extension_plan_snapshot();

    assert_eq!(before.registration_count(), 1);
    assert_eq!(after.registration_count(), 2);
    assert!(!Arc::ptr_eq(&before, &after));
}

#[test]
fn world_runtime_extension_callback_can_publish_a_new_generation() {
    let driver = Arc::new(WorldDriver::default());
    let reentrant_driver = Arc::clone(&driver);
    driver
        .install_world_runtime_extension_plan(
            WorldRuntimeExtensionPlan::from_registrations([
                WorldRuntimeExtensionRegistration::new("reentrant", move |_| {
                    let guard = reentrant_driver
                        .runtime_extensions
                        .try_lock()
                        .map_err(|_| {
                            WorldRuntimeExtensionError::new(
                                "world extension callback ran while the driver lock was held",
                            )
                        })?;
                    drop(guard);
                    reentrant_driver
                        .install_world_runtime_extension_plan(extension_plan("during.apply"))
                        .map_err(|error| WorldRuntimeExtensionError::new(error.to_string()))
                }),
            ])
            .expect("reentrant extension plan"),
        )
        .expect("initial extension plan");

    let mut world = World::new();
    driver
        .apply_world_runtime_extensions(&mut world)
        .expect("extension callback can publish a successor generation");

    assert_eq!(
        driver
            .runtime_extension_plan_snapshot()
            .registration_count(),
        2
    );
}

#[test]
fn world_runtime_extension_callbacks_overlap_across_independent_worlds() {
    const WORLD_COUNT: usize = 4;

    let driver = Arc::new(WorldDriver::default());
    let callbacks_in_flight = Arc::new(AtomicUsize::new(0));
    let peak_callbacks_in_flight = Arc::new(AtomicUsize::new(0));
    let callbacks_in_flight_for_registration = Arc::clone(&callbacks_in_flight);
    let peak_callbacks_in_flight_for_registration = Arc::clone(&peak_callbacks_in_flight);
    driver
        .install_world_runtime_extension_plan(
            WorldRuntimeExtensionPlan::from_registrations([
                WorldRuntimeExtensionRegistration::new("concurrent", move |_| {
                    let in_flight =
                        callbacks_in_flight_for_registration.fetch_add(1, Ordering::SeqCst) + 1;
                    peak_callbacks_in_flight_for_registration
                        .fetch_max(in_flight, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(20));
                    callbacks_in_flight_for_registration.fetch_sub(1, Ordering::SeqCst);
                    Ok(())
                }),
            ])
            .expect("concurrent extension plan"),
        )
        .expect("initial extension plan");

    let start = Arc::new(Barrier::new(WORLD_COUNT));
    let workers = (0..WORLD_COUNT)
        .map(|_| {
            let driver = Arc::clone(&driver);
            let start = Arc::clone(&start);
            thread::spawn(move || {
                let mut world = World::new();
                start.wait();
                driver
                    .apply_world_runtime_extensions(&mut world)
                    .expect("world extension callback applies");
            })
        })
        .collect::<Vec<_>>();
    for worker in workers {
        worker.join().expect("world extension worker completes");
    }

    assert!(
        peak_callbacks_in_flight.load(Ordering::SeqCst) > 1,
        "independent Worlds must not serialize callbacks behind the driver lock"
    );
}

fn extension_plan(key: &str) -> WorldRuntimeExtensionPlan {
    WorldRuntimeExtensionPlan::from_registrations([WorldRuntimeExtensionRegistration::new(
        key,
        |_| Ok(()),
    )])
    .expect("unique extension plan")
}
