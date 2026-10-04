use super::{
    ProductExitClass, ProductFailureLedger, ProductFailureSeverity, ProductHostPhase,
    ProductProcessExitCode, ProductShutdownCoordinator, ProductShutdownPhaseDisposition,
    ProductShutdownTransitionError, ProductTerminalOutcome, ProductTerminalReason,
    ProductTerminalSecondary, PRODUCT_FAILURE_LEDGER_CAPACITY, PRODUCT_FAILURE_MESSAGE_BYTES,
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
fn product_exit_classes_use_the_versioned_portable_host_registry() {
    let cases = [
        (ProductExitClass::Success, 0),
        (ProductExitClass::UnclassifiedFailure, 1),
        (ProductExitClass::UsageFailure, 2),
        (ProductExitClass::CapabilityFailure, 3),
        (ProductExitClass::ConfigFailure, 4),
        (ProductExitClass::StartupFailure, 5),
        (ProductExitClass::RuntimeFailure, 6),
        (ProductExitClass::ShutdownFailure, 7),
        (ProductExitClass::ForcedTermination, 8),
    ];
    for (class, code) in cases {
        assert_eq!(ProductProcessExitCode::from_class(class).code(), code);
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
fn terminal_outcome_preserves_primary_and_orders_bounded_secondary_failures() {
    let mut outcome =
        ProductTerminalOutcome::host(ProductExitClass::RuntimeFailure, "runtime_runner_failed")
            .with_identity(42, Some(7));
    outcome.observe_report_write(false);
    outcome.observe_diagnostic_log_shutdown(false);
    outcome.observe_report_write(false);

    assert_eq!(outcome.primary().exit_code().code(), 6);
    assert_eq!(outcome.exit_code().code(), 6);
    assert_eq!(
        outcome.secondary(),
        &[
            ProductTerminalSecondary::ReportWriteFailed,
            ProductTerminalSecondary::DiagnosticLogShutdownFailed,
        ]
    );
    let receipt = serde_json::to_value(outcome.receipt()).unwrap();
    assert_eq!(receipt["schema_version"], 1);
    assert_eq!(receipt["attempt"], 42);
    assert_eq!(receipt["generation"], 7);
    assert_eq!(receipt["primary"]["source"], "host");
    assert_eq!(receipt["primary"]["class"], "runtime_failure");
    assert_eq!(receipt["primary_exit_code"], 6);
    assert_eq!(receipt["process_exit_code"], 6);
    assert_eq!(receipt["report_write"], "failed");
    assert_eq!(receipt["diagnostic_log_shutdown"], "failed");
}

#[test]
fn successful_primary_with_failed_teardown_projects_shutdown_without_rewriting_primary() {
    let mut outcome = ProductTerminalOutcome::host(ProductExitClass::Success, "completed");
    outcome.observe_diagnostic_log_shutdown(false);
    assert_eq!(outcome.primary().exit_code().code(), 0);
    assert_eq!(outcome.exit_code().code(), 7);
    let receipt = serde_json::to_value(outcome.receipt()).unwrap();
    assert_eq!(receipt["primary_exit_code"], 0);
    assert_eq!(receipt["process_exit_code"], 7);
    assert!(receipt["attempt"].is_null());
    assert_eq!(receipt["shutdown_durability"], "unknown");
}

#[test]
fn commandlet_primary_keeps_raw_code_even_when_it_collides_with_host_registry() {
    let mut outcome = ProductTerminalOutcome::commandlet(3);
    outcome.observe_ipc_write(false);
    assert_eq!(outcome.primary().exit_code().code(), 3);
    assert_eq!(outcome.exit_code().code(), 3);
    let receipt = serde_json::to_value(outcome.receipt()).unwrap();
    assert_eq!(receipt["primary"]["source"], "commandlet");
    assert_eq!(receipt["primary"]["code"], 3);
    assert_eq!(receipt["process_exit_code"], 3);
}

#[test]
fn terminal_secondary_ledger_and_failed_status_remain_bounded_after_retries() {
    let mut outcome = ProductTerminalOutcome::host(ProductExitClass::Success, "completed");
    for _ in 0..10 {
        outcome.observe_shutdown_durability(false);
        outcome.observe_report_write(false);
        outcome.observe_ipc_write(false);
        outcome.observe_profiling_flush(false);
        outcome.observe_diagnostic_log_shutdown(false);
    }
    outcome.observe_report_write(true);
    outcome.observe_diagnostic_log_shutdown(true);

    assert_eq!(outcome.secondary().len(), 5);
    assert_eq!(outcome.exit_code().code(), 7);
    let receipt = serde_json::to_value(outcome.receipt()).unwrap();
    assert_eq!(receipt["report_write"], "failed");
    assert_eq!(receipt["diagnostic_log_shutdown"], "failed");
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
