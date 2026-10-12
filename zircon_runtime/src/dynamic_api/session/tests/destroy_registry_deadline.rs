use super::super::allocation_registry::RuntimeAllocationCensus;
use super::*;

#[derive(Clone, Copy)]
enum RegistryKind {
    Session,
    Allocation,
}

fn hold_registry_lock(
    kind: RegistryKind,
    max_hold: Duration,
) -> (mpsc::Sender<()>, thread::JoinHandle<()>) {
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let holder = thread::spawn(move || {
        let wait_for_release = move || {
            entered_tx
                .send(())
                .expect("registry holder should report entry");
            let _ = release_rx.recv_timeout(max_hold);
        };
        match kind {
            RegistryKind::Session => {
                super::super::session_store::with_registry_lock_for_test(wait_for_release)
            }
            RegistryKind::Allocation => {
                super::super::allocation_registry::with_registry_lock_for_test(wait_for_release)
            }
        }
    });
    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("registry holder should acquire its lock");
    (release_tx, holder)
}

fn session_with_census() -> (
    zircon_runtime_interface::ZrRuntimeSessionHandle,
    zircon_runtime_interface::ZrRuntimeAllocationId,
) {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    let allocation = register_runtime_allocation(
        handle,
        RuntimeAllocationKind::Accessibility,
        vec![1_u8, 2, 3, 4],
    )
    .expect("runtime allocation")
    .allocation;
    (handle, allocation)
}

#[test]
fn destroy_registry_deadline_bounds_initial_lookup_and_retains_census() {
    let _serial = REGISTRY_TIMING_TEST_SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (handle, allocation) = session_with_census();
    assert_eq!(
        release_runtime_allocation(handle, allocation).status_code(),
        ZrStatusCode::Ok
    );
    let before = allocation_census(handle);
    assert_eq!(before.outstanding_allocations, 0);
    assert_eq!(before.high_water_allocations, 1);

    let (release, holder) = hold_registry_lock(RegistryKind::Session, Duration::from_millis(300));
    let started = Instant::now();
    let status = destroy_session_slot_with_timeout(handle, Duration::from_millis(25)).status_code();
    let elapsed = started.elapsed();
    let _ = release.send(());
    holder.join().expect("registry holder should finish");

    assert_eq!(status, ZrStatusCode::Error);
    assert!(
        elapsed < Duration::from_millis(200),
        "lookup exceeded destroy budget: {elapsed:?}"
    );
    assert!(!session_is_closing(handle));
    assert_eq!(allocation_census(handle), before);
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}

#[test]
fn destroy_registry_deadline_bounds_allocation_check_and_keeps_release_retryable() {
    let _serial = REGISTRY_TIMING_TEST_SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (handle, allocation) = session_with_census();
    let before = allocation_census(handle);
    let (release, holder) =
        hold_registry_lock(RegistryKind::Allocation, Duration::from_millis(300));
    let started = Instant::now();
    let status = destroy_session_slot_with_timeout(handle, Duration::from_millis(25)).status_code();
    let elapsed = started.elapsed();
    let _ = release.send(());
    holder.join().expect("registry holder should finish");

    assert_eq!(status, ZrStatusCode::Error);
    assert!(
        elapsed < Duration::from_millis(200),
        "allocation check exceeded destroy budget: {elapsed:?}"
    );
    assert!(session_is_closing(handle));
    assert_eq!(allocation_census(handle), before);
    assert_eq!(
        release_runtime_allocation(handle, allocation).status_code(),
        ZrStatusCode::Ok
    );
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}

fn joined_slot_with_empty_census() -> (
    zircon_runtime_interface::ZrRuntimeSessionHandle,
    Arc<super::super::session_slot::SessionSlot>,
    super::super::allocation_registry::RuntimeAllocationCensus,
) {
    let (handle, allocation) = session_with_census();
    assert_eq!(
        release_runtime_allocation(handle, allocation).status_code(),
        ZrStatusCode::Ok
    );
    let before = allocation_census(handle);
    let slot = super::super::session_store::session_slot_for_test(handle)
        .expect("session slot remains registered");
    assert!(slot.begin_close());
    slot.frame_activity().disable_wake_entries();
    assert!(slot.wait_for_actions(Duration::from_secs(1)));
    assert!(slot
        .frame_activity()
        .wait_for_wake_callbacks(Duration::from_secs(1)));
    assert!(matches!(
        slot.shutdown_until(Instant::now() + Duration::from_secs(5)),
        super::super::session_owner::OwnerShutdownReceipt::Joined
    ));
    (handle, slot, before)
}

fn assert_final_registry_lock_timeout(kind: RegistryKind) {
    let _serial = REGISTRY_TIMING_TEST_SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (handle, slot, before) = joined_slot_with_empty_census();
    let (release, holder) = hold_registry_lock(kind, Duration::from_millis(300));
    let started = Instant::now();
    let removed = super::super::session_store::remove_session_and_census_until(
        handle,
        &slot,
        started + Duration::from_millis(25),
    );
    let elapsed = started.elapsed();
    let _ = release.send(());
    holder.join().expect("registry holder should finish");

    assert!(
        !removed,
        "final removal must not commit while a registry is held"
    );
    assert!(
        elapsed < Duration::from_millis(200),
        "final removal exceeded destroy budget: {elapsed:?}"
    );
    assert!(session_is_closing(handle));
    assert_eq!(allocation_census(handle), before);
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
    assert_eq!(
        allocation_census(handle),
        RuntimeAllocationCensus::default()
    );
}

#[test]
fn destroy_registry_deadline_bounds_final_session_removal() {
    assert_final_registry_lock_timeout(RegistryKind::Session);
}

#[test]
fn destroy_registry_deadline_bounds_final_allocation_removal() {
    assert_final_registry_lock_timeout(RegistryKind::Allocation);
}

#[test]
fn destroy_registry_deadline_concurrent_final_removal_is_idempotent() {
    let _serial = REGISTRY_TIMING_TEST_SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (handle, slot, before) = joined_slot_with_empty_census();
    let first_slot = Arc::clone(&slot);
    let second_slot = Arc::clone(&slot);
    let first = thread::spawn(move || {
        super::super::session_store::remove_session_and_census_until(
            handle,
            &first_slot,
            Instant::now() + Duration::from_secs(1),
        )
    });
    let second = thread::spawn(move || {
        super::super::session_store::remove_session_and_census_until(
            handle,
            &second_slot,
            Instant::now() + Duration::from_secs(1),
        )
    });
    assert!(first.join().expect("first final-removal worker"));
    assert!(second.join().expect("second final-removal worker"));
    assert_eq!(
        allocation_census(handle),
        RuntimeAllocationCensus::default()
    );
    assert_eq!(
        destroy_session_slot(handle).status_code(),
        ZrStatusCode::NotFound
    );
}
