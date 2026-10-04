use super::{editor_process_exit_code, editor_process_teardown_diagnostic};

fn host_success() -> zircon_app::ProductTerminalOutcome {
    zircon_app::ProductTerminalOutcome::host(
        zircon_app::ProductExitClass::Success,
        "editor_completed",
    )
}

#[test]
fn completed_process_log_shutdown_preserves_the_editor_exit_code() {
    assert_eq!(
        editor_process_exit_code(
            Ok::<_, std::io::Error>(zircon_app::ProductTerminalOutcome::commandlet(7)),
            true,
        )
        .exit_code()
        .code(),
        7
    );
}

#[test]
fn process_log_shutdown_failure_is_secondary_and_fails_a_successful_editor_exit() {
    assert_eq!(
        editor_process_exit_code(Ok::<_, std::io::Error>(host_success()), false)
            .exit_code()
            .code(),
        7
    );
}

#[test]
fn failed_log_shutdown_preserves_nonzero_commandlet_primary() {
    let outcome = editor_process_exit_code(
        Ok::<_, std::io::Error>(zircon_app::ProductTerminalOutcome::commandlet(73)),
        false,
    );
    assert_eq!(outcome.primary().exit_code().code(), 73);
    assert_eq!(outcome.exit_code().code(), 73);
    assert_eq!(outcome.secondary().len(), 1);
}

#[test]
fn successful_commandlet_retains_commandlet_provenance_after_process_projection() {
    let outcome = editor_process_exit_code(
        Ok::<_, std::io::Error>(zircon_app::ProductTerminalOutcome::commandlet(0)),
        true,
    );
    let receipt = serde_json::to_value(outcome.receipt()).unwrap();
    assert_eq!(receipt["primary"]["source"], "commandlet");
    assert_eq!(receipt["primary_exit_code"], 0);
    assert_eq!(outcome.exit_code().code(), 0);
}

#[test]
fn successful_commandlet_with_failed_log_shutdown_keeps_its_primary_origin() {
    let outcome = editor_process_exit_code(
        Ok::<_, std::io::Error>(zircon_app::ProductTerminalOutcome::commandlet(0)),
        false,
    );
    assert_eq!(
        outcome.primary(),
        zircon_app::ProductTerminalPrimary::Commandlet { code: 0 }
    );
    assert_eq!(outcome.exit_code().code(), 7);
    assert_eq!(outcome.secondary().len(), 1);
}

#[test]
fn successful_plugin_list_route_retains_commandlet_provenance() {
    let outcome = zircon_app::EntryRunner::run_editor_with_args_terminal(["--run", "plugin-list"])
        .expect("plugin-list should finish without loading the GUI host");
    assert_eq!(
        outcome.primary(),
        zircon_app::ProductTerminalPrimary::Commandlet { code: 0 }
    );
}

#[test]
fn untyped_editor_runner_failure_keeps_generic_code_when_log_shutdown_fails() {
    let outcome = editor_process_exit_code(Err(std::io::Error::other("failed")), false);
    assert_eq!(outcome.primary().exit_code().code(), 1);
    assert_eq!(outcome.exit_code().code(), 1);
    assert_eq!(outcome.secondary().len(), 1);
}

#[test]
fn teardown_diagnostic_records_completed_exit_code() {
    assert_eq!(
        editor_process_teardown_diagnostic(&Ok::<_, ()>(host_success())),
        "editor_process_teardown_complete result=completed exit_code=0"
    );
}

#[test]
fn teardown_diagnostic_records_failed_exit_code() {
    assert_eq!(
        editor_process_teardown_diagnostic(&Err::<zircon_app::ProductTerminalOutcome, ()>(())),
        "editor_process_teardown_complete result=failed exit_code=1"
    );
}
