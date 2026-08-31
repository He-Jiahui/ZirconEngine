use super::{
    ProductExitClass, ProductFailureLedger, ProductFailureSeverity, ProductHostPhase,
    ProductProcessExitCode, ProductShutdownCoordinator, ProductShutdownPhaseDisposition,
    ProductShutdownTransitionError, ProductTerminalReason, PRODUCT_FAILURE_LEDGER_CAPACITY,
    PRODUCT_FAILURE_MESSAGE_BYTES,
};

#[test]
fn failure_ledger_is_ordered_bounded_and_reports_suppression() {
    let ledger = ProductFailureLedger::default();

    for index in 0..(PRODUCT_FAILURE_LEDGER_CAPACITY + 2) {
        ledger.record(
            ProductHostPhase::Running,
            ProductFailureSeverity::Terminal,
            "runtime_frame",
            format!("failure-{index}"),
        );
    }

    let report = ledger.snapshot();
    assert_eq!(report.records().len(), PRODUCT_FAILURE_LEDGER_CAPACITY);
    assert_eq!(report.suppressed_count(), 2);
    assert_eq!(report.primary().unwrap().sequence(), 0);
    assert_eq!(report.secondary().last().unwrap().sequence(), 15);
    assert_eq!(report.primary().unwrap().message(), "failure-0");
}

#[test]
fn failure_ledger_truncates_messages_on_a_utf8_boundary() {
    let ledger = ProductFailureLedger::default();
    ledger.record(
        ProductHostPhase::DestroyingRuntime,
        ProductFailureSeverity::Emergency,
        "runtime_session",
        "界".repeat(PRODUCT_FAILURE_MESSAGE_BYTES),
    );

    let report = ledger.snapshot();
    let message = report.primary().unwrap().message();
    assert!(message.len() <= PRODUCT_FAILURE_MESSAGE_BYTES);
    assert!(message.ends_with("..."));
}

#[test]
fn failure_ledger_escapes_single_line_record_delimiters_before_bounding() {
    let ledger = ProductFailureLedger::default();
    ledger.record(
        ProductHostPhase::FlushingDiagnostics,
        ProductFailureSeverity::Terminal,
        "runtime_play_report",
        "alpha=\r\nbeta | gamma\t",
    );

    let report = ledger.snapshot();
    assert_eq!(
        report.primary().unwrap().message(),
        "alpha\\=\\r\\nbeta \\| gamma\\t"
    );
}

#[test]
fn terminal_reason_maps_to_a_portable_exit_class_without_numeric_policy() {
    assert_eq!(
        ProductTerminalReason::Completed.exit_class(),
        ProductExitClass::Success
    );
    assert_eq!(
        ProductTerminalReason::StartupFailed.exit_class(),
        ProductExitClass::StartupFailure
    );
    assert_eq!(
        ProductTerminalReason::RuntimeFailed.exit_class(),
        ProductExitClass::RuntimeFailure
    );
    assert_eq!(
        ProductTerminalReason::ShutdownFailed.exit_class(),
        ProductExitClass::ShutdownFailure
    );
}

#[test]
fn product_exit_classes_use_only_the_portable_success_and_generic_failure_codes() {
    assert_eq!(
        ProductProcessExitCode::from_class(ProductExitClass::Success).code(),
        0
    );
    for class in [
        ProductExitClass::StartupFailure,
        ProductExitClass::RuntimeFailure,
        ProductExitClass::ShutdownFailure,
        ProductExitClass::ForcedTermination,
    ] {
        assert_eq!(ProductProcessExitCode::from_class(class).code(), 1);
    }
}

#[test]
fn explicit_command_exit_codes_remain_distinct_from_host_failure_classification() {
    assert_eq!(
        ProductProcessExitCode::from_code(0),
        ProductProcessExitCode::Success
    );
    assert_eq!(ProductProcessExitCode::from_code(73).code(), 73);
    assert!(ProductProcessExitCode::from_code(73).is_failure());
}

#[test]
fn shutdown_coordinator_keeps_the_first_reason_and_advances_monotonically() {
    let coordinator = ProductShutdownCoordinator::default();
    coordinator.mark_running().unwrap();
    coordinator
        .request_stop(ProductTerminalReason::WindowClosed)
        .unwrap();
    coordinator
        .request_stop(ProductTerminalReason::RuntimeFailed)
        .unwrap();

    for phase in [
        ProductHostPhase::Draining,
        ProductHostPhase::ReleasingPlatform,
        ProductHostPhase::DestroyingRuntime,
        ProductHostPhase::DeactivatingModules,
        ProductHostPhase::FlushingDiagnostics,
        ProductHostPhase::Exited,
    ] {
        coordinator.advance_to(phase).unwrap();
        coordinator.advance_to(phase).unwrap();
    }

    let snapshot = coordinator.snapshot();
    assert_eq!(snapshot.phase(), ProductHostPhase::Exited);
    assert_eq!(
        snapshot.terminal_reason(),
        Some(ProductTerminalReason::WindowClosed)
    );
    assert_eq!(snapshot.transitions().len(), ProductHostPhase::COUNT - 1);
}

#[test]
fn shutdown_coordinator_rejects_skipped_or_backward_phases() {
    let coordinator = ProductShutdownCoordinator::default();

    assert_eq!(
        coordinator.advance_to(ProductHostPhase::Draining),
        Err(ProductShutdownTransitionError::InvalidTransition {
            from: ProductHostPhase::Composing,
            to: ProductHostPhase::Draining,
        })
    );
    coordinator
        .request_stop(ProductTerminalReason::StartupFailed)
        .unwrap();
    assert_eq!(
        coordinator.advance_to(ProductHostPhase::Running),
        Err(ProductShutdownTransitionError::InvalidTransition {
            from: ProductHostPhase::Quiescing,
            to: ProductHostPhase::Running,
        })
    );
}

#[test]
fn shutdown_coordinator_records_phase_disposition_without_claiming_missing_owners() {
    let coordinator = ProductShutdownCoordinator::default();
    coordinator.mark_running().unwrap();
    coordinator
        .request_stop(ProductTerminalReason::Completed)
        .unwrap();
    coordinator
        .advance_to_with_disposition(
            ProductHostPhase::Draining,
            ProductShutdownPhaseDisposition::LegacyCombined,
        )
        .unwrap();
    coordinator
        .advance_to_with_disposition(
            ProductHostPhase::Draining,
            ProductShutdownPhaseDisposition::Executed,
        )
        .unwrap();
    coordinator
        .advance_to_with_disposition(
            ProductHostPhase::ReleasingPlatform,
            ProductShutdownPhaseDisposition::NoOwner,
        )
        .unwrap();
    coordinator
        .advance_to_with_disposition(
            ProductHostPhase::DestroyingRuntime,
            ProductShutdownPhaseDisposition::LegacyCombined,
        )
        .unwrap();

    let snapshot = coordinator.snapshot();
    let transitions = snapshot.transitions();
    assert_eq!(
        transitions[0].disposition(),
        ProductShutdownPhaseDisposition::Executed
    );
    assert_eq!(
        transitions[1].disposition(),
        ProductShutdownPhaseDisposition::Executed
    );
    assert_eq!(
        transitions[2].disposition(),
        ProductShutdownPhaseDisposition::LegacyCombined
    );
    assert_eq!(
        transitions[3].disposition(),
        ProductShutdownPhaseDisposition::NoOwner
    );
    assert_eq!(
        transitions[4].disposition(),
        ProductShutdownPhaseDisposition::LegacyCombined
    );
}

#[test]
fn startup_rollback_can_record_that_no_running_owner_required_quiescing() {
    let coordinator = ProductShutdownCoordinator::default();
    coordinator
        .request_stop_with_disposition(
            ProductTerminalReason::StartupFailed,
            ProductShutdownPhaseDisposition::NoOwner,
        )
        .unwrap();

    let snapshot = coordinator.snapshot();
    assert_eq!(snapshot.phase(), ProductHostPhase::Quiescing);
    assert_eq!(snapshot.transitions().len(), 1);
    assert_eq!(
        snapshot.transitions()[0].disposition(),
        ProductShutdownPhaseDisposition::NoOwner
    );
}
