use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use super::super::super::NativePluginLoader;
use super::super::{
    NativePluginDiscoveryRefreshService, NativePluginDiscoveryRefreshTerminal,
    NativePluginDiscoveryRefreshTicket,
};
use super::support::{
    root, test_budget, wait_for_terminal, BarrierFailureCollector, BlockingCollector,
    SequenceCollector,
};

#[test]
fn cancel_racing_with_failure_publication_never_records_cancelled_with_a_failure() {
    for attempt in 0..32 {
        let (started_sender, started_receiver) = mpsc::sync_channel(1);
        let barrier = Arc::new(Barrier::new(3));
        let collector = Arc::new(BarrierFailureCollector::new(
            started_sender,
            Arc::clone(&barrier),
        ));
        let service = NativePluginDiscoveryRefreshService::new(collector, test_budget());
        let root = root(&format!("failure-race-{attempt}"));
        let ticket = service.submit(root.clone());
        started_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("collector start");

        let cancel_ticket = ticket.clone();
        let cancel_barrier = Arc::clone(&barrier);
        let canceller = std::thread::spawn(move || {
            cancel_barrier.wait();
            cancel_ticket.cancel()
        });
        barrier.wait();
        let cancelled = canceller.join().expect("cancel thread");
        std::thread::sleep(Duration::from_millis(10));

        match ticket.terminal() {
            Some(NativePluginDiscoveryRefreshTerminal::Cancelled) => {
                assert!(cancelled);
                assert!(service.last_failure(&root).is_none());
            }
            Some(NativePluginDiscoveryRefreshTerminal::Failed(_)) => {
                assert!(!cancelled);
                assert!(service.last_failure(&root).is_some());
            }
            terminal => panic!("unexpected terminal state: {terminal:?}"),
        }
    }
}

#[test]
fn cancellation_and_shutdown_terminal_observers_run_once() {
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let collector = Arc::new(BlockingCollector::new(started_sender));
    let service = NativePluginDiscoveryRefreshService::new(collector, test_budget());
    let ticket = service.submit(root("terminal-once"));
    let observed = Arc::new(AtomicUsize::new(0));
    let observed_for_callback = Arc::clone(&observed);
    assert!(ticket.on_terminal(move |_| {
        observed_for_callback.fetch_add(1, Ordering::SeqCst);
    }));

    started_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("collector start");
    ticket.cancel();
    service.shutdown();

    assert_eq!(observed.load(Ordering::SeqCst), 1);
    assert!(matches!(
        ticket.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Cancelled)
    ));
}

#[test]
fn terminal_observer_admission_remains_bounded_after_completion() {
    let mut budget = test_budget();
    budget.max_terminal_observers = 1;
    let service =
        NativePluginDiscoveryRefreshService::new(Arc::new(SequenceCollector::new()), budget);
    let ticket = service.submit(root("post-terminal-observer-budget"));
    wait_for_terminal(&ticket);

    let observed = Arc::new(AtomicUsize::new(0));
    let first = Arc::clone(&observed);
    assert!(ticket.on_terminal(move |_| {
        first.fetch_add(1, Ordering::SeqCst);
    }));
    assert!(!ticket.on_terminal(|_| {}));
    assert_eq!(observed.load(Ordering::SeqCst), 1);
}

#[test]
fn ticket_wait_uses_terminal_notification_without_polling() {
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let service = NativePluginDiscoveryRefreshService::new(
        Arc::new(BlockingCollector::new(started_sender)),
        test_budget(),
    );
    let ticket = service.submit(root("terminal-notification"));
    let waiting_ticket = ticket.clone();
    let waiter = std::thread::spawn(move || waiting_ticket.wait_terminal());

    started_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("collector start");
    ticket.cancel();

    assert!(matches!(
        waiter.join().expect("terminal waiter"),
        NativePluginDiscoveryRefreshTerminal::Cancelled
    ));
    assert!(include_str!("../ticket.rs").contains("Condvar"));
}

#[test]
fn queued_ticket_deadline_terminalizes_without_a_collector_worker() {
    let ticket = NativePluginDiscoveryRefreshTicket::new(
        root("queued-deadline"),
        1,
        Instant::now() + Duration::from_millis(10),
        1,
    );

    assert!(matches!(
        ticket.wait_terminal(),
        NativePluginDiscoveryRefreshTerminal::DeadlineExceeded
    ));
}

#[test]
fn terminal_observer_reentry_projects_without_waiting_on_collector_lane() {
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let barrier = Arc::new(Barrier::new(2));
    let service = NativePluginDiscoveryRefreshService::new(
        Arc::new(BarrierFailureCollector::new(
            started_sender,
            Arc::clone(&barrier),
        )),
        test_budget(),
    );
    let ticket = service.submit(root("terminal-observer-reentry"));
    let (report_sender, report_receiver) = mpsc::sync_channel(1);
    let reentry_root = std::path::PathBuf::from("C:/native-plugin-tests/observer-reentry");
    assert!(ticket.on_terminal(move |_| {
        report_sender
            .send(NativePluginLoader.discover(&reentry_root))
            .expect("observer result receiver remains available");
    }));

    started_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("collector start");
    barrier.wait();

    let report = report_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("terminal observer must not wait on the collector I/O lane");
    assert!(report.diagnostics().iter().any(|diagnostic| {
        diagnostic.contains("cannot synchronously establish a root from its collector I/O lane")
    }));
    assert!(matches!(
        ticket.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Failed(_))
    ));
}

#[test]
fn failure_terminal_is_reserved_before_recording_the_last_failure() {
    let source = include_str!("../service.rs");
    let locked_completion = source
        .split_once("let mut state = shared")
        .expect("completion continues to hold the service state lock")
        .1
        .split_once("    };\n\n    if let Some(delivery)")
        .expect("completion releases the state lock before observer delivery")
        .0;
    assert!(
        locked_completion.contains("ticket.reserve_terminal(terminal)")
            && locked_completion
                .contains("root_state.last_failure = Some(RefreshFailure { error });"),
        "a cancellation racing with failure completion must resolve before failure state commits under the service state lock"
    );
}

#[test]
fn io_lane_covers_terminal_observer_delivery() {
    let source = include_str!("../service.rs");
    let lane_guard = source
        .split_once("NATIVE_PLUGIN_DISCOVERY_IO_LANE.with")
        .expect("scheduled discovery work marks its I/O lane")
        .1;
    let completion = lane_guard
        .find("complete_generation(&task_shared")
        .expect("scheduled work completes its generation");
    let reset = lane_guard
        .find("in_lane.set(previous)")
        .expect("scheduled work restores its lane marker");
    assert!(
        completion < reset,
        "terminal observers must run while collector-I/O re-entry is explicitly nonblocking"
    );
    let fallback = source
        .split_once("handle.on_terminal(move ||")
        .expect("scheduler fallback observer")
        .1;
    assert!(
        fallback.contains("NATIVE_PLUGIN_DISCOVERY_IO_LANE.with")
            && fallback.contains("complete_generation(")
            && fallback.contains("in_lane.set(previous)"),
        "scheduler fallback delivery must preserve the collector-I/O re-entry guard"
    );
}

#[test]
fn queued_deadline_is_retired_as_a_deadline_failure() {
    let source = include_str!("../service.rs");
    let terminal_branch = source
        .split_once("else if let Some(existing_terminal) = ticket.terminal()")
        .expect("completed tickets retain their terminal reason")
        .1
        .split_once("} else if ticket.cancellation().is_explicitly_cancelled()")
        .expect("explicit cancellation remains distinct")
        .0;
    assert!(
        terminal_branch.contains("NativePluginDiscoveryRefreshTerminal::DeadlineExceeded")
            && terminal_branch.contains("root_state.last_failure"),
        "a queued deadline must be recorded as its actual terminal failure"
    );
}
