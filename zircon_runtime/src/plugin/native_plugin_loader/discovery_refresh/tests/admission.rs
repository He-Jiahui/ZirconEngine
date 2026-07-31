use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};

use super::super::{
    NativePluginDiscoveryRefreshError, NativePluginDiscoveryRefreshInput,
    NativePluginDiscoveryRefreshService, NativePluginDiscoveryRefreshTerminal,
};
use super::support::{
    root, test_budget, wait_for_terminal, BlockingCollector, InputBarrierFailureCollector,
    SequenceCollector,
};

#[test]
fn refresh_submit_returns_before_the_collector_finishes() {
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let collector = Arc::new(BlockingCollector::new(started_sender));
    let service = NativePluginDiscoveryRefreshService::new(collector, test_budget());

    let ticket = service.submit(root("nonblocking"));

    assert!(
        !ticket.is_complete(),
        "submit must only admit and schedule discovery work"
    );
    assert_eq!(
        started_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("collector must run on the bounded I/O lane"),
        1
    );

    ticket.cancel();
    assert!(matches!(
        ticket.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Cancelled)
    ));
}

#[test]
fn newest_generation_supersedes_active_work_and_only_publishes_latest_snapshot() {
    let (started_sender, started_receiver) = mpsc::sync_channel(2);
    let collector = Arc::new(BlockingCollector::new(started_sender));
    let service = NativePluginDiscoveryRefreshService::new(collector, test_budget());
    let root = root("coalesce");

    let first = service.submit(root.clone());
    assert_eq!(
        started_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("first collection start"),
        1
    );
    let stale_pending = service.submit(root.clone());
    let latest = service.submit(root.clone());

    assert!(matches!(
        first.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Superseded { .. })
    ));
    assert!(matches!(
        stale_pending.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Superseded { .. })
    ));
    assert_eq!(
        started_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("newest coalesced generation start"),
        3
    );
    wait_for_terminal(&latest);

    let snapshot = service.snapshot(&root).expect("latest snapshot");
    assert_eq!(snapshot.generation(), 3);
    assert_eq!(snapshot.diagnostics(), &["generation 3".to_owned()]);
}

#[test]
fn elapsed_deadline_terminalizes_a_running_refresh_without_publishing() {
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let collector = Arc::new(BlockingCollector::new(started_sender));
    let mut budget = test_budget();
    budget.deadline = Duration::from_millis(1);
    let service = NativePluginDiscoveryRefreshService::new(collector, budget);
    let root = root("deadline");

    let ticket = service.submit(root.clone());
    started_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("collector start");
    wait_for_terminal(&ticket);

    assert!(matches!(
        ticket.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::DeadlineExceeded)
    ));
    assert!(service.snapshot(&root).is_none());
    assert!(matches!(
        service.last_failure(&root).as_deref(),
        Some(NativePluginDiscoveryRefreshError::DeadlineExceeded)
    ));
}

#[test]
fn unrepresentable_deadline_is_rejected_without_starting_the_collector() {
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let collector = Arc::new(BlockingCollector::new(started_sender));
    let mut budget = test_budget();
    budget.deadline = Duration::MAX;
    let service = NativePluginDiscoveryRefreshService::new(collector, budget);

    let ticket = service.submit(root("deadline-overflow"));

    assert!(matches!(
        ticket.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Rejected { .. })
    ));
    assert!(started_receiver
        .recv_timeout(Duration::from_millis(20))
        .is_err());
}

#[test]
fn bounded_root_admission_rejects_new_roots_without_evicting_a_snapshot() {
    let mut budget = test_budget();
    budget.max_roots = 1;
    let service =
        NativePluginDiscoveryRefreshService::new(Arc::new(SequenceCollector::new()), budget);
    let first_root = root("first");
    let first = service.submit(first_root.clone());
    wait_for_terminal(&first);

    let rejected = service.submit(root("second"));

    assert!(matches!(
        rejected.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Rejected { .. })
    ));
    assert!(service.snapshot(&first_root).is_some());
}

#[test]
fn same_root_distinct_inputs_are_isolated_for_admission_and_failure_projection() {
    let (started_sender, started_receiver) = mpsc::sync_channel(2);
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let collector = Arc::new(InputBarrierFailureCollector::new(
        started_sender,
        Arc::clone(&release),
    ));
    let pool = TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(2));
    let service =
        NativePluginDiscoveryRefreshService::with_pool(collector.clone(), pool, test_budget());
    let root = root("input-isolation");
    let selection_input = NativePluginDiscoveryRefreshInput::load_manifest(
        "C:/native-plugin-tests/input-isolation".into(),
    );

    let scan = service.submit(root.clone());
    let selection = service.submit_with_input(root.clone(), selection_input.clone());

    let first_input = started_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("root scan collector start");
    let second_input = match started_receiver.recv_timeout(Duration::from_secs(1)) {
        Ok(input) => input,
        Err(error) => {
            collector.release();
            panic!("load manifest collector must start independently: {error}");
        }
    };
    assert!(matches!(
        (first_input, second_input),
        (
            NativePluginDiscoveryRefreshInput::RootScan,
            NativePluginDiscoveryRefreshInput::LoadManifest { .. }
        ) | (
            NativePluginDiscoveryRefreshInput::LoadManifest { .. },
            NativePluginDiscoveryRefreshInput::RootScan
        )
    ));

    collector.release();
    wait_for_terminal(&scan);
    wait_for_terminal(&selection);

    assert!(matches!(
        scan.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Failed(_))
    ));
    assert!(matches!(
        selection.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Failed(_))
    ));
    assert_eq!(
        service.last_failure(&root).as_deref(),
        Some(&NativePluginDiscoveryRefreshError::collector(
            "root scan failure"
        ))
    );
    assert_eq!(
        service.last_failure_for(&root, &selection_input).as_deref(),
        Some(&NativePluginDiscoveryRefreshError::collector(
            "load manifest failure"
        ))
    );
}
