use std::sync::Arc;
use std::time::Instant;

use crate::core::framework::platform::{
    PlatformHostBackend, PlatformHostBackendKind, PlatformHostFailureReason,
    PlatformHostLifecycleState,
};
use crate::platform::test_support::platform_driver;

use super::fixtures::{
    observed_host_evidence, RecordingPlatformHostBackend, RejectingPlatformHostBackend,
};

#[test]
fn platform_host_becomes_ready_only_after_its_installed_instance_is_observed() {
    let driver = platform_driver();
    let backend = Arc::new(RecordingPlatformHostBackend::new());

    assert_eq!(
        driver.platform_host_snapshot().lifecycle(),
        PlatformHostLifecycleState::Uninstalled
    );
    assert!(!driver.platform_host_snapshot().is_ready());

    let starting = driver
        .install_platform_host(backend)
        .expect("an uninstalled driver accepts its first host bridge");
    assert_eq!(starting.lifecycle(), PlatformHostLifecycleState::Starting);
    assert!(!starting.is_ready());
    assert_eq!(
        starting
            .descriptor()
            .expect("installed host has a descriptor")
            .backend(),
        PlatformHostBackendKind::Winit
    );

    let ready = driver
        .publish_platform_host_ready(
            starting.instance().expect("installed host has an instance"),
            observed_host_evidence(),
        )
        .expect("the installed instance may publish observed readiness");
    assert_eq!(ready.lifecycle(), PlatformHostLifecycleState::Ready);
    assert!(ready.is_ready());
    assert_eq!(
        ready
            .evidence()
            .expect("ready host requires observed evidence")
            .backend_version(),
        Some("test-host")
    );
}

#[test]
fn platform_host_quiesce_is_single_flight_and_requires_a_matching_terminal_receipt() {
    let driver = platform_driver();
    let backend = Arc::new(RecordingPlatformHostBackend::new());
    let starting = driver
        .install_platform_host(Arc::clone(&backend))
        .expect("host installs");
    driver
        .publish_platform_host_ready(
            starting.instance().expect("installed host has an instance"),
            observed_host_evidence(),
        )
        .expect("host becomes ready");

    let first = driver
        .request_platform_host_quiesce(Instant::now())
        .expect("ready host accepts a quiesce request");
    let second = driver
        .request_platform_host_quiesce(Instant::now())
        .expect("a duplicate quiesce request returns the in-flight operation");
    assert_eq!(first.operation(), second.operation());
    assert_eq!(backend.request_count(), 1);
    assert_eq!(
        driver.platform_host_snapshot().lifecycle(),
        PlatformHostLifecycleState::Quiescing
    );

    let quiesced = driver
        .publish_platform_host_quiesced(first)
        .expect("matching host receipt closes the in-flight operation");
    assert_eq!(quiesced.lifecycle(), PlatformHostLifecycleState::Quiesced);
    assert_eq!(quiesced.active_operation(), None);
    assert_eq!(quiesced.evidence(), None);
    assert!(quiesced
        .terminal()
        .expect("quiesce produces a receipt")
        .is_quiesced());
}

#[test]
fn stale_platform_host_receipts_cannot_mutate_a_restarted_host() {
    let driver = platform_driver();
    let first = driver
        .install_platform_host(Arc::new(RecordingPlatformHostBackend::new()))
        .expect("first host installs");
    let first_instance = first.instance().expect("installed host has an instance");
    driver
        .publish_platform_host_failed(first_instance, PlatformHostFailureReason::OwnerExited)
        .expect("owner loss is terminal");

    let restarted = driver
        .install_platform_host(Arc::new(RecordingPlatformHostBackend::new()))
        .expect("failed host can be replaced by a new instance");
    let restarted_instance = restarted
        .instance()
        .expect("restarted host has an instance");
    assert_ne!(first_instance, restarted_instance);
    assert!(driver
        .publish_platform_host_ready(first_instance, observed_host_evidence())
        .is_err());
    assert_eq!(
        driver.platform_host_snapshot().instance(),
        Some(restarted_instance)
    );
    assert_eq!(
        driver.platform_host_snapshot().lifecycle(),
        PlatformHostLifecycleState::Starting
    );
}

#[test]
fn platform_host_failure_revokes_ready_without_leaving_a_live_backend_bridge() {
    let driver = platform_driver();
    let starting = driver
        .install_platform_host(Arc::new(RecordingPlatformHostBackend::new()))
        .expect("host installs");
    let instance = starting.instance().expect("installed host has an instance");
    driver
        .publish_platform_host_ready(instance, observed_host_evidence())
        .expect("host becomes ready");

    let failed = driver
        .publish_platform_host_failed(instance, PlatformHostFailureReason::OwnerExited)
        .expect("owner loss transitions the host to failed");

    assert_eq!(failed.lifecycle(), PlatformHostLifecycleState::Failed);
    assert!(!failed.is_ready());
    assert_eq!(failed.evidence(), None);
    assert!(failed
        .terminal()
        .expect("failure has a terminal receipt")
        .is_failed());
    assert!(driver
        .request_platform_host_quiesce(Instant::now())
        .is_err());
}

#[test]
fn rejected_platform_host_quiesce_request_transitions_to_failed() {
    let driver = platform_driver();
    let starting = driver
        .install_platform_host(Arc::new(RejectingPlatformHostBackend))
        .expect("host installs");
    let instance = starting.instance().expect("installed host has an instance");
    driver
        .publish_platform_host_ready(instance, observed_host_evidence())
        .expect("host becomes ready");

    assert!(driver
        .request_platform_host_quiesce(Instant::now())
        .is_err());
    let failed = driver.platform_host_snapshot();

    assert_eq!(failed.lifecycle(), PlatformHostLifecycleState::Failed);
    assert!(!failed.is_ready());
    assert!(failed
        .terminal()
        .expect("rejected request has a terminal failure")
        .is_failed());
}
