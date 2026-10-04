use super::*;

#[test]
fn job_terminal_observer_registered_before_completion_runs_once() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let (observed_tx, observed_rx) = bounded::<()>(1);
    let observer_runs = Arc::new(AtomicUsize::new(0));
    let handle =
        scheduler.schedule(move || release_rx.recv_timeout(Duration::from_secs(2)).unwrap());

    let observer_runs_for_callback = Arc::clone(&observer_runs);
    handle.on_terminal(move || {
        observer_runs_for_callback.fetch_add(1, Ordering::SeqCst);
        observed_tx.send(()).unwrap();
    });

    assert_eq!(observer_runs.load(Ordering::SeqCst), 0);
    release_tx.send(()).unwrap();
    observed_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    handle.wait();

    assert_eq!(observer_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn job_terminal_observer_registered_after_completion_runs_once() {
    let scheduler = single_worker_scheduler();
    let handle = scheduler.schedule(|| {});
    handle.wait();
    let observer_runs = Arc::new(AtomicUsize::new(0));

    let observer_runs_for_callback = Arc::clone(&observer_runs);
    handle.on_terminal(move || {
        observer_runs_for_callback.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(observer_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn multiple_job_terminal_observers_each_run_exactly_once() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let (observed_tx, observed_rx) = bounded::<()>(3);
    let observer_runs = Arc::new(AtomicUsize::new(0));
    let handle =
        scheduler.schedule(move || release_rx.recv_timeout(Duration::from_secs(2)).unwrap());

    for _ in 0..3 {
        let observer_runs_for_callback = Arc::clone(&observer_runs);
        let observed_tx = observed_tx.clone();
        handle.on_terminal(move || {
            observer_runs_for_callback.fetch_add(1, Ordering::SeqCst);
            observed_tx.send(()).unwrap();
        });
    }

    release_tx.send(()).unwrap();
    for _ in 0..3 {
        observed_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    }
    handle.wait();
    handle.wait();

    assert_eq!(observer_runs.load(Ordering::SeqCst), 3);
}

#[test]
fn job_terminal_observer_panic_is_contained_and_recorded() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let (survivor_tx, survivor_rx) = bounded::<()>(1);
    let handle =
        scheduler.schedule(move || release_rx.recv_timeout(Duration::from_secs(2)).unwrap());

    handle.on_terminal(|| panic!("terminal observer failure"));
    handle.on_terminal(move || survivor_tx.send(()).unwrap());
    release_tx.send(()).unwrap();
    survivor_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    handle.wait();

    assert_eq!(handle.terminal_observer_panic_count(), 1);
    handle.on_terminal(|| panic!("late terminal observer failure"));
    assert_eq!(handle.terminal_observer_panic_count(), 2);
    handle.wait();
}

#[test]
fn job_terminal_observer_preserves_dependency_continuation_order() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(2),
    ));
    let (release_tx, release_rx) = bounded::<()>(0);
    let dependency =
        scheduler.schedule(move || release_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    let (dependent_tx, dependent_rx) = bounded::<()>(1);
    let dependent = scheduler.schedule_after(&[dependency.clone()], move || {
        dependent_tx.send(()).unwrap();
    });
    let (observer_tx, observer_rx) = bounded::<()>(1);

    dependency.on_terminal(move || {
        dependent_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("dependency continuation must launch before terminal observers run");
        observer_tx.send(()).unwrap();
    });

    release_tx.send(()).unwrap();
    observer_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    dependency.wait();
    dependent.wait();

    assert_eq!(dependency.terminal_observer_panic_count(), 0);
}

#[test]
fn job_terminal_observer_can_reenter_handle_accessors() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let handle =
        scheduler.schedule(move || release_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    let handle_for_callback = handle.clone();
    let observer_runs = Arc::new(AtomicUsize::new(0));
    let observer_runs_for_callback = Arc::clone(&observer_runs);
    let (observed_tx, observed_rx) = bounded::<()>(1);

    handle.on_terminal(move || {
        assert!(handle_for_callback.is_complete());
        observer_runs_for_callback.fetch_add(1, Ordering::SeqCst);
        let observer_runs_for_nested = Arc::clone(&observer_runs_for_callback);
        handle_for_callback.on_terminal(move || {
            observer_runs_for_nested.fetch_add(1, Ordering::SeqCst);
        });
        observed_tx.send(()).unwrap();
    });

    release_tx.send(()).unwrap();
    observed_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    handle.wait();

    assert_eq!(observer_runs.load(Ordering::SeqCst), 2);
    assert_eq!(handle.terminal_observer_panic_count(), 0);
}
