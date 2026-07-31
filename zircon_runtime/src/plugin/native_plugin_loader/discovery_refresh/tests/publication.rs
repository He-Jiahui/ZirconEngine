use std::sync::Arc;

use super::super::{
    NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError,
    NativePluginDiscoveryRefreshService, NativePluginDiscoveryRefreshTerminal,
};
use super::support::{
    root, test_budget, wait_for_terminal, BudgetFailureCollector, SequenceCollector,
};

#[test]
fn published_ticket_cannot_be_cancelled_after_its_snapshot_is_committed() {
    let service =
        NativePluginDiscoveryRefreshService::new(Arc::new(SequenceCollector::new()), test_budget());
    let root = root("published-terminal");
    let ticket = service.submit(root.clone());
    wait_for_terminal(&ticket);

    assert!(!ticket.cancel());
    assert!(matches!(
        ticket.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Published(_))
    ));
    assert_eq!(
        service
            .snapshot(&root)
            .expect("published snapshot")
            .generation(),
        ticket.generation()
    );
    assert_eq!(
        service
            .snapshot(&root)
            .expect("published snapshot")
            .input_identity(),
        &NativePluginDiscoveryInputIdentity::new("fixture-generation-1").expect("fixture identity")
    );
}

#[test]
fn empty_input_identity_is_rejected_before_a_payload_can_be_published() {
    assert!(matches!(
        NativePluginDiscoveryInputIdentity::new(" "),
        Err(NativePluginDiscoveryRefreshError::InvalidInputIdentity)
    ));
}

#[test]
fn refresh_failure_preserves_the_last_good_snapshot() {
    let collector = Arc::new(SequenceCollector::new());
    let service = NativePluginDiscoveryRefreshService::new(collector, test_budget());
    let root = root("last-good");

    let first = service.submit(root.clone());
    wait_for_terminal(&first);
    let first_snapshot = service.snapshot(&root).expect("first snapshot");

    let second = service.submit(root.clone());
    wait_for_terminal(&second);

    assert!(matches!(
        second.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Failed(_))
    ));
    assert_eq!(
        service
            .snapshot(&root)
            .expect("last-good snapshot remains")
            .generation(),
        first_snapshot.generation()
    );
}

#[test]
fn resource_budget_failures_preserve_the_last_good_snapshot() {
    let service = NativePluginDiscoveryRefreshService::new(
        Arc::new(BudgetFailureCollector::new()),
        test_budget(),
    );
    let root = root("last-good-budget");

    let first = service.submit(root.clone());
    wait_for_terminal(&first);
    let first_snapshot = service.snapshot(&root).expect("first snapshot");

    let rejected = service.submit(root.clone());
    wait_for_terminal(&rejected);

    assert!(matches!(
        rejected.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Failed(_))
    ));
    assert_eq!(
        service
            .snapshot(&root)
            .expect("last-good snapshot remains")
            .generation(),
        first_snapshot.generation()
    );
}
