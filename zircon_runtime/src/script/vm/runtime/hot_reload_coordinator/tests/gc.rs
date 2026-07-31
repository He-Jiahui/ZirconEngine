use super::*;

#[test]
fn gc_pending_queue_deduplicates_without_linear_queue_search() {
    let mut pending = PendingGcSlots::default();
    let slot = PluginSlotId::new(7);

    assert!(pending.push_back(slot));
    assert!(!pending.push_back(slot));
    assert_eq!(pending.pop_front(), Some(slot));
    assert!(pending.push_front(slot));
    assert!(!pending.push_front(slot));
    assert!(pending.remove(slot));
    assert_eq!(pending.pop_front(), None);

    let source = include_str!("../../hot_reload_coordinator.rs");
    assert!(source.contains("members: HashSet<PluginSlotId>"));
    assert!(!source.contains("pending.contains(&slot)"));
}

#[test]
fn gc_step_respects_frame_budget() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let backend = GcRecordingBackend {
        calls: Arc::clone(&calls),
    };
    let coordinator = HotReloadCoordinator::new();
    let host = test_host_context();
    let first = coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package("first", 4, VmPluginGarbageCollectionMode::Cooperative, None),
            &host,
        )
        .unwrap();
    let second = coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package(
                "second",
                8,
                VmPluginGarbageCollectionMode::Cooperative,
                None,
            ),
            &host,
        )
        .unwrap();
    coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package("third", 1, VmPluginGarbageCollectionMode::Cooperative, None),
            &host,
        )
        .unwrap();

    let report = coordinator
        .gc_step(VmGcBudget {
            max_micros_per_frame: 10,
        })
        .unwrap();

    assert_eq!(
        calls.lock().unwrap().as_slice(),
        &[(first, 10), (second, 6)]
    );
    assert_eq!(report.slots.len(), 2);
    assert_eq!(report.pause_micros, 12);
    assert_eq!(report.overrun_micros, 2);
    assert_eq!(report.slots[1].budget_micros, 6);
}

#[test]
fn gc_step_only_host_steps_cooperative_active_slots_in_slot_order() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let backend = GcRecordingBackend {
        calls: Arc::clone(&calls),
    };
    let coordinator = HotReloadCoordinator::new();
    let host = test_host_context();
    coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package(
                "backend",
                1,
                VmPluginGarbageCollectionMode::BackendManaged,
                None,
            ),
            &host,
        )
        .unwrap();
    coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package("disabled", 1, VmPluginGarbageCollectionMode::Disabled, None),
            &host,
        )
        .unwrap();
    let cooperative = coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package(
                "cooperative",
                1,
                VmPluginGarbageCollectionMode::Cooperative,
                None,
            ),
            &host,
        )
        .unwrap();

    let report = coordinator.gc_step(VmGcBudget::default()).unwrap();

    assert_eq!(calls.lock().unwrap().as_slice(), &[(cooperative, 1_000)]);
    assert_eq!(report.slots[0].slot, cooperative);
}

#[test]
fn gc_step_honors_interval_frames_without_wall_clock_timing() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let backend = GcRecordingBackend {
        calls: Arc::clone(&calls),
    };
    let coordinator = HotReloadCoordinator::new();
    let host = test_host_context();
    let slot = coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package(
                "interval",
                1,
                VmPluginGarbageCollectionMode::Cooperative,
                Some(2),
            ),
            &host,
        )
        .unwrap();

    assert!(coordinator
        .gc_step(VmGcBudget::default())
        .unwrap()
        .slots
        .is_empty());
    assert_eq!(
        coordinator.gc_step(VmGcBudget::default()).unwrap().slots[0].slot,
        slot
    );
    assert!(coordinator
        .gc_step(VmGcBudget::default())
        .unwrap()
        .slots
        .is_empty());
    assert_eq!(calls.lock().unwrap().len(), 1);
}

#[test]
fn panicking_gc_backend_restores_instance_and_pending_work() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let backend = GcRecordingBackend {
        calls: Arc::clone(&calls),
    };
    let coordinator = HotReloadCoordinator::new();
    let host = test_host_context();
    let slot = coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package(
                "panic-once",
                1,
                VmPluginGarbageCollectionMode::Cooperative,
                None,
            ),
            &host,
        )
        .unwrap();

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = coordinator.gc_step(VmGcBudget::default());
    }));
    assert!(panic.is_err());
    assert_eq!(
        coordinator.slot(slot).unwrap().state,
        VmPluginSlotState::Active
    );

    let report = coordinator.gc_step(VmGcBudget::default()).unwrap();
    assert_eq!(report.slots[0].slot, slot);
    assert_eq!(calls.lock().unwrap().len(), 2);
}

#[test]
fn gc_pending_fifo_prevents_full_budget_slot_starvation() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let backend = GcRecordingBackend {
        calls: Arc::clone(&calls),
    };
    let coordinator = HotReloadCoordinator::new();
    let host = test_host_context();
    let first = coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package(
                "first",
                10,
                VmPluginGarbageCollectionMode::Cooperative,
                None,
            ),
            &host,
        )
        .unwrap();
    let second = coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package(
                "second",
                1,
                VmPluginGarbageCollectionMode::Cooperative,
                None,
            ),
            &host,
        )
        .unwrap();
    let third = coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package("third", 1, VmPluginGarbageCollectionMode::Cooperative, None),
            &host,
        )
        .unwrap();
    let budget = VmGcBudget {
        max_micros_per_frame: 10,
    };

    assert_eq!(coordinator.gc_step(budget).unwrap().slots[0].slot, first);
    let second_frame = coordinator.gc_step(budget).unwrap();

    assert_eq!(second_frame.slots[0].slot, second);
    assert_eq!(second_frame.slots[1].slot, third);
}

#[test]
fn interval_due_work_remains_pending_past_due_frame() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let backend = GcRecordingBackend {
        calls: Arc::clone(&calls),
    };
    let coordinator = HotReloadCoordinator::new();
    let host = test_host_context();
    coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package(
                "every-frame",
                10,
                VmPluginGarbageCollectionMode::Cooperative,
                None,
            ),
            &host,
        )
        .unwrap();
    let interval = coordinator
        .load_package(
            "gc-recording",
            &backend,
            gc_test_package(
                "interval",
                1,
                VmPluginGarbageCollectionMode::Cooperative,
                Some(2),
            ),
            &host,
        )
        .unwrap();
    let budget = VmGcBudget {
        max_micros_per_frame: 10,
    };

    coordinator.gc_step(budget).unwrap();
    coordinator.gc_step(budget).unwrap();
    let third_frame = coordinator.gc_step(budget).unwrap();

    assert_eq!(third_frame.frame_index, 3);
    assert_eq!(third_frame.slots[0].slot, interval);
}

#[test]
fn concurrent_gc_step_calls_are_serialized_across_backend_entry() {
    let (entered_tx, entered_rx) = mpsc::sync_channel(2);
    let (release_tx, release_rx) = mpsc::sync_channel(2);
    let calls = Arc::new(AtomicUsize::new(0));
    let backend = Arc::new(BlockingGcBackend {
        entered: entered_tx,
        release: Arc::new(Mutex::new(release_rx)),
        calls: Arc::clone(&calls),
    });
    let coordinator = Arc::new(HotReloadCoordinator::new());
    let host = test_host_context();
    coordinator
        .load_package(
            "blocking-gc",
            backend.as_ref(),
            gc_test_package(
                "every-frame-overlap",
                1,
                VmPluginGarbageCollectionMode::Cooperative,
                None,
            ),
            &host,
        )
        .unwrap();

    let first = {
        let coordinator = Arc::clone(&coordinator);
        std::thread::spawn(move || coordinator.gc_step(VmGcBudget::default()))
    };
    entered_rx.recv().unwrap();
    let (second_started_tx, second_started_rx) = mpsc::sync_channel(1);
    let second = {
        let coordinator = Arc::clone(&coordinator);
        std::thread::spawn(move || {
            second_started_tx.send(()).unwrap();
            coordinator.gc_step(VmGcBudget::default())
        })
    };
    second_started_rx.recv().unwrap();
    release_tx.send(()).unwrap();
    entered_rx.recv().unwrap();
    release_tx.send(()).unwrap();

    let first_report = first.join().unwrap().unwrap();
    let second_report = second.join().unwrap().unwrap();
    assert_eq!(first_report.frame_index, 1);
    assert_eq!(first_report.slots.len(), 1);
    assert_eq!(second_report.frame_index, 2);
    assert_eq!(second_report.slots.len(), 1);
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
}
