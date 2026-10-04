use super::*;

#[test]
fn job_diagnostics_track_schedule_complete_and_wait_times() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let dependency = scheduler.schedule(move || {
        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    });
    let after = scheduler.schedule_after(&[dependency], || {});
    let release_thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        release_tx.send(()).unwrap();
    });

    after.wait();
    release_thread.join().unwrap();

    let report = scheduler.diagnostic_report();
    assert_eq!(report.scheduled, 2);
    assert_eq!(report.completed, 2);
    assert!(
        report.dependency_wait_ms > 0.0,
        "dependency wait should capture time spent waiting for prerequisites"
    );
    assert!(
        report.explicit_wait_ms > 0.0,
        "explicit wait should capture handle synchronization time"
    );
    let formatted = report.format_diagnostics();
    assert!(formatted.contains("tasks.scheduled=2"));
    assert!(formatted.contains("tasks.completed=2"));

    let mut store = DiagnosticStore::default();
    scheduler.record_diagnostics(&mut store, 7);
    let snapshot = store.snapshot();

    assert_eq!(
        diagnostic_current(&snapshot, TASKS_SCHEDULED_DIAGNOSTIC),
        2.0
    );
    assert_eq!(
        diagnostic_current(&snapshot, TASKS_COMPLETED_DIAGNOSTIC),
        2.0
    );
    assert!(diagnostic_current(&snapshot, TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC) > 0.0);
    assert!(diagnostic_current(&snapshot, TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC) > 0.0);
}

#[test]
fn task_diagnostics_track_ready_queue_active_and_queue_wait() {
    let scheduler = single_worker_scheduler();
    let (started_tx, started_rx) = bounded::<()>(1);
    let (release_tx, release_rx) = bounded::<()>(0);
    let first = scheduler.schedule(move || {
        started_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    });
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();

    let second = scheduler.schedule(|| {});
    let saturated = scheduler.diagnostic_report();
    assert_eq!(saturated.scheduled, 2);
    assert_eq!(saturated.queued, 1);
    assert_eq!(saturated.active, 1);
    assert_eq!(saturated.completed, 0);

    release_tx.send(()).unwrap();
    scheduler.wait_all(&[first, second]);

    let drained = scheduler.diagnostic_report();
    assert_eq!(drained.queued, 0);
    assert_eq!(drained.active, 0);
    assert_eq!(drained.completed, 2);
    assert_eq!(drained.queue_wait_samples, 2);
    assert!(drained.queue_wait_ms > 0.0);

    let mut store = DiagnosticStore::default();
    scheduler.record_diagnostics(&mut store, 9);
    let snapshot = store.snapshot();
    assert_eq!(diagnostic_current(&snapshot, TASKS_QUEUED_DIAGNOSTIC), 0.0);
    assert_eq!(diagnostic_current(&snapshot, TASKS_ACTIVE_DIAGNOSTIC), 0.0);
    assert_eq!(
        diagnostic_current(&snapshot, TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC),
        2.0
    );
    assert!(diagnostic_current(&snapshot, TASKS_QUEUE_WAIT_MS_DIAGNOSTIC) > 0.0);
}

#[test]
fn task_diagnostics_track_dependency_waiting_through_release_and_cancellation() {
    let scheduler = single_worker_scheduler();
    let (started_tx, started_rx) = bounded::<()>(1);
    let (release_tx, release_rx) = bounded::<()>(0);
    let dependency = scheduler.schedule(move || {
        started_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    });
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let dependent = scheduler.schedule_after(&[dependency.clone()], || {});

    let waiting = scheduler.diagnostic_report();
    assert_eq!(waiting.scheduled, 2);
    assert_eq!(waiting.completed, 0);
    assert_eq!(waiting.dependency_waiting, 1);
    assert_eq!(waiting.queued, 0);
    assert_eq!(waiting.active, 1);
    assert_eq!(
        waiting.scheduled,
        waiting.completed + waiting.dependency_waiting + waiting.queued + waiting.active
    );

    release_tx.send(()).unwrap();
    scheduler.wait_all(&[dependency, dependent]);
    let released = scheduler.diagnostic_report();
    assert_eq!(released.dependency_waiting, 0);
    assert_eq!(released.completed, 2);

    let cancelled_scheduler = single_worker_scheduler();
    let failed = cancelled_scheduler.schedule(|| panic!("dependency failure"));
    let cancelled = cancelled_scheduler.schedule_after(&[failed], || {
        panic!("cancelled dependent must not run");
    });
    assert!(catch_unwind(AssertUnwindSafe(|| cancelled.wait())).is_err());
    let cancelled_report = cancelled_scheduler.diagnostic_report();
    assert_eq!(cancelled_report.dependency_waiting, 0);
    assert_eq!(cancelled_report.cancelled, 1);
    assert_eq!(
        cancelled_report.queue_wait_samples + cancelled_report.cancelled,
        cancelled_report.completed + cancelled_report.active
    );
    assert_eq!(
        cancelled_report.scheduled,
        cancelled_report.completed
            + cancelled_report.dependency_waiting
            + cancelled_report.queued
            + cancelled_report.active
    );

    let mut store = DiagnosticStore::default();
    cancelled_scheduler.record_diagnostics(&mut store, 10);
    assert_eq!(
        diagnostic_current(&store.snapshot(), TASKS_DEPENDENCY_WAITING_DIAGNOSTIC),
        0.0
    );
}

#[test]
fn task_diagnostics_queue_pressure_matrix_drains_without_gauge_leaks() {
    for worker_count in [1, 2, 4] {
        let scheduler = JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::compute().with_worker_threads(worker_count),
        ))
        .with_diagnostics();
        let (started_tx, started_rx) = bounded::<()>(worker_count);
        let (release_tx, release_rx) = unbounded::<()>();
        let mut handles = Vec::with_capacity(worker_count * 3);

        for _ in 0..worker_count {
            let started_tx = started_tx.clone();
            let release_rx = release_rx.clone();
            handles.push(scheduler.schedule(move || {
                started_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            }));
        }
        for _ in 0..worker_count {
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        }

        handles.extend((0..worker_count * 2).map(|_| scheduler.schedule(|| {})));
        let saturated = scheduler.diagnostic_report();
        assert_eq!(saturated.active, worker_count as u64);
        assert_eq!(saturated.queued, (worker_count * 2) as u64);
        assert_eq!(saturated.queue_wait_samples, worker_count as u64);

        for _ in 0..worker_count {
            release_tx.send(()).unwrap();
        }
        scheduler.wait_all(&handles);

        let drained = scheduler.diagnostic_report();
        assert_eq!(drained.scheduled, (worker_count * 3) as u64);
        assert_eq!(drained.completed, (worker_count * 3) as u64);
        assert_eq!(drained.queue_wait_samples, (worker_count * 3) as u64);
        assert_eq!(drained.queued, 0);
        assert_eq!(drained.active, 0);
        assert!(drained.queue_wait_ms > 0.0);
    }
}

#[test]
fn task_diagnostics_reports_conserved_lifecycle_snapshots_during_transitions() {
    const TASK_COUNT: usize = 128;
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(4),
    ))
    .with_diagnostics();
    let (release_tx, release_rx) = unbounded::<()>();
    let handles = (0..TASK_COUNT)
        .map(|_| {
            let release_rx = release_rx.clone();
            scheduler.schedule(move || {
                release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            })
        })
        .collect::<Vec<_>>();
    let releaser = std::thread::spawn(move || {
        for _ in 0..TASK_COUNT {
            release_tx.send(()).unwrap();
            std::thread::sleep(Duration::from_millis(1));
        }
    });

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut saw_intermediate_completion = false;
    loop {
        let report_started_at = Instant::now();
        let report = scheduler.diagnostic_report();
        assert!(
            report_started_at.elapsed() < Duration::from_millis(250),
            "diagnostic_report must make bounded progress while workers transition"
        );
        assert_eq!(
            report.scheduled,
            report.completed + report.dependency_waiting + report.queued + report.active,
            "a stable lifecycle snapshot must not lose or double-count admitted tasks"
        );
        assert_eq!(
            report.queue_wait_samples + report.cancelled,
            report.completed + report.active,
            "started samples plus never-started cancellations must conserve terminal work"
        );
        saw_intermediate_completion |= report.completed > 0 && report.completed < TASK_COUNT as u64;
        if report.completed == TASK_COUNT as u64 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "diagnostic reports did not advance to the terminal snapshot"
        );
        std::thread::yield_now();
    }

    releaser.join().unwrap();
    scheduler.wait_all(&handles);
    assert!(
        saw_intermediate_completion,
        "reporting must expose at least one stable in-flight lifecycle snapshot"
    );
}

#[test]
fn task_diagnostics_keep_conserved_snapshots_during_concurrent_admission() {
    const PRODUCER_COUNT: usize = 4;
    const TASKS_PER_PRODUCER: usize = 128;
    const TASK_COUNT: usize = PRODUCER_COUNT * TASKS_PER_PRODUCER;

    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(PRODUCER_COUNT),
    ))
    .with_diagnostics();
    let (release_tx, release_rx) = unbounded::<()>();
    let (admission_midpoint_tx, admission_midpoint_rx) = bounded::<()>(PRODUCER_COUNT);
    let (resume_admission_tx, resume_admission_rx) = bounded::<()>(PRODUCER_COUNT);
    let admission_start = Arc::new(Barrier::new(PRODUCER_COUNT + 1));
    let producers_finished = Arc::new(AtomicUsize::new(0));
    let producers = (0..PRODUCER_COUNT)
        .map(|_| {
            let scheduler = scheduler.clone();
            let release_rx = release_rx.clone();
            let admission_midpoint_tx = admission_midpoint_tx.clone();
            let resume_admission_rx = resume_admission_rx.clone();
            let admission_start = Arc::clone(&admission_start);
            let producers_finished = Arc::clone(&producers_finished);
            std::thread::spawn(move || {
                admission_start.wait();
                let handles = (0..TASKS_PER_PRODUCER)
                    .map(|index| {
                        let release_rx = release_rx.clone();
                        let handle = scheduler.schedule(move || {
                            release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                        });
                        if index + 1 == TASKS_PER_PRODUCER / 2 {
                            admission_midpoint_tx.send(()).unwrap();
                            resume_admission_rx
                                .recv_timeout(Duration::from_secs(2))
                                .unwrap();
                        }
                        std::thread::yield_now();
                        handle
                    })
                    .collect::<Vec<_>>();
                producers_finished.fetch_add(1, Ordering::Release);
                handles
            })
        })
        .collect::<Vec<_>>();

    admission_start.wait();
    for _ in 0..PRODUCER_COUNT {
        admission_midpoint_rx
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    let midpoint_scheduled = (PRODUCER_COUNT * (TASKS_PER_PRODUCER / 2)) as u64;
    let report = loop {
        let report = scheduler.diagnostic_report();
        if report.scheduled == midpoint_scheduled {
            break report;
        }
        assert!(
            Instant::now() < deadline,
            "midpoint reporting did not observe every admitted task"
        );
        std::thread::yield_now();
    };
    assert_eq!(report.scheduled, midpoint_scheduled);
    assert_eq!(report.completed, 0);
    assert_eq!(
        report.scheduled,
        report.completed + report.dependency_waiting + report.queued + report.active,
        "an aggregate snapshot must not span concurrent admission and worker transitions"
    );
    assert_eq!(
        report.queue_wait_samples + report.cancelled,
        report.completed + report.active,
        "queue samples and terminal work must remain conserved while producers submit"
    );
    assert!(
        producers_finished.load(Ordering::Acquire) < PRODUCER_COUNT,
        "the regression must sample before all concurrent producers finish admission"
    );
    for _ in 0..PRODUCER_COUNT {
        resume_admission_tx.send(()).unwrap();
    }

    let mut reports = 1;
    loop {
        let report = scheduler.diagnostic_report();
        assert_eq!(
            report.scheduled,
            report.completed + report.dependency_waiting + report.queued + report.active,
            "an aggregate snapshot must not span concurrent admission and worker transitions"
        );
        assert_eq!(
            report.queue_wait_samples + report.cancelled,
            report.completed + report.active,
            "queue samples and terminal work must remain conserved while producers submit"
        );
        reports += 1;
        if producers_finished.load(Ordering::Acquire) == PRODUCER_COUNT {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "concurrent producers did not finish their bounded admission phase"
        );
        std::thread::yield_now();
    }
    assert!(
        reports > 0,
        "reporting must sample the concurrent admission phase"
    );

    let handles = producers
        .into_iter()
        .flat_map(|producer| producer.join().unwrap())
        .collect::<Vec<_>>();
    for _ in 0..TASK_COUNT {
        release_tx.send(()).unwrap();
    }
    scheduler.wait_all(&handles);

    let terminal = scheduler.diagnostic_report();
    assert_eq!(terminal.scheduled, TASK_COUNT as u64);
    assert_eq!(terminal.completed, TASK_COUNT as u64);
    assert_eq!(terminal.queued, 0);
    assert_eq!(terminal.active, 0);
}

#[test]
fn worker_side_wait_is_reported_as_explicit_wait() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(2),
    ))
    .with_diagnostics();
    let (started_tx, started_rx) = bounded::<()>(1);
    let (release_tx, release_rx) = bounded::<()>(0);
    let dependency = scheduler.schedule(move || {
        started_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    });
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();

    let waiter = scheduler.schedule(move || dependency.wait());
    std::thread::sleep(Duration::from_millis(20));
    release_tx.send(()).unwrap();

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while !waiter.is_complete() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(waiter.is_complete());

    let report = scheduler.diagnostic_report();
    assert!(report.explicit_wait_ms > 0.0);
    assert!(!report.format_diagnostics().contains("main_thread_wait"));
}

#[test]
fn task_diagnostics_distinguish_panics_from_dependency_cancellation() {
    let scheduler = single_worker_scheduler();
    let dependency = scheduler.schedule(|| panic!("dependency failure"));
    let dependent = scheduler.schedule_after(&[dependency], || {
        panic!("dependency cancellation must prevent this task from running");
    });

    let result = catch_unwind(AssertUnwindSafe(|| dependent.wait()));
    assert!(result.is_err());

    let report = scheduler.diagnostic_report();
    assert_eq!(report.scheduled, 2);
    assert_eq!(report.completed, 2);
    assert_eq!(report.panicked, 1);
    assert_eq!(report.cancelled, 1);
    assert_eq!(report.queued, 0);
    assert_eq!(report.active, 0);

    let mut store = DiagnosticStore::default();
    scheduler.record_diagnostics(&mut store, 11);
    let snapshot = store.snapshot();
    assert_eq!(
        diagnostic_current(&snapshot, TASKS_PANICKED_DIAGNOSTIC),
        1.0
    );
    assert_eq!(
        diagnostic_current(&snapshot, TASKS_CANCELLED_DIAGNOSTIC),
        1.0
    );
}
