use super::super::{
    editor_operation_startup_error, editor_startup_argument_error, editor_startup_argument_summary,
    finish_editor_host, finish_editor_operation, EditorCliOperationRequest,
};
use zircon_editor::core::editor_operation::EditorOperationControlRequest;

#[test]
fn editor_cli_operation_parser_accepts_operation_args_and_headless() {
    let request = EditorCliOperationRequest::parse([
        "--operation".to_string(),
        "window.layout.reset".to_string(),
        "--args".to_string(),
        r#"{"source":"ci"}"#.to_string(),
        "--headless".to_string(),
    ])
    .unwrap()
    .unwrap();

    assert_eq!(
        request.operation_id.as_ref().unwrap().as_str(),
        "window.layout.reset"
    );
    assert_eq!(request.arguments["source"], "ci");
    assert!(request.headless);
}

#[test]
fn editor_cli_operation_parser_accepts_operation_group() {
    let request = EditorCliOperationRequest::parse([
        "--operation".to_string(),
        "viewport.transform.apply".to_string(),
        "--operation-group".to_string(),
        "Viewport.TransformDrag.42".to_string(),
        "--headless".to_string(),
    ])
    .unwrap()
    .unwrap();

    assert_eq!(
        request.operation_group.as_deref(),
        Some("Viewport.TransformDrag.42")
    );

    let EditorOperationControlRequest::InvokeOperation(invocation) =
        request.into_control_request().unwrap()
    else {
        panic!("expected InvokeOperation request");
    };
    assert_eq!(
        invocation.operation_group.as_deref(),
        Some("Viewport.TransformDrag.42")
    );
}

#[test]
fn editor_cli_operation_parser_rejects_operation_group_without_operation() {
    let error = EditorCliOperationRequest::parse([
        "--operation-group".to_string(),
        "Viewport.TransformDrag.42".to_string(),
        "--headless".to_string(),
    ])
    .unwrap_err();

    assert_eq!(error.to_string(), "--operation-group requires --operation");
}

#[test]
fn editor_cli_operation_parser_rejects_empty_operation_group() {
    let error = EditorCliOperationRequest::parse([
        "--operation".to_string(),
        "viewport.transform.apply".to_string(),
        "--operation-group".to_string(),
        "  ".to_string(),
        "--headless".to_string(),
    ])
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "--operation-group requires a non-empty group id"
    );
}

#[test]
fn editor_cli_operation_parser_rejects_args_without_operation() {
    let error = EditorCliOperationRequest::parse([
        "--args".to_string(),
        r#"{"source":"ci"}"#.to_string(),
        "--headless".to_string(),
    ])
    .unwrap_err();

    assert_eq!(error.to_string(), "--args requires --operation");
}

#[test]
fn editor_cli_operation_parser_rejects_null_args_without_operation() {
    let error = EditorCliOperationRequest::parse([
        "--args".to_string(),
        "null".to_string(),
        "--headless".to_string(),
    ])
    .unwrap_err();

    assert_eq!(error.to_string(), "--args requires --operation");
}

#[test]
fn editor_cli_operation_parser_rejects_headless_without_control_request() {
    let error = EditorCliOperationRequest::parse(["--headless".to_string()]).unwrap_err();

    assert_eq!(
        error.to_string(),
        "--headless requires --operation, --list-operations, or --operation-history"
    );
}

#[test]
fn editor_cli_operation_parser_rejects_operation_mixed_with_list_operations() {
    let error = EditorCliOperationRequest::parse([
        "--operation".to_string(),
        "window.layout.reset".to_string(),
        "--list-operations".to_string(),
        "--headless".to_string(),
    ])
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "--operation, --list-operations, and --operation-history are mutually exclusive"
    );
}

#[test]
fn editor_operation_startup_conflict_reports_an_actionable_product_error() {
    let args = vec![
        "--operation".to_string(),
        "window.layout.reset".to_string(),
        "--list-operations".to_string(),
        "--headless".to_string(),
    ];
    let source = EditorCliOperationRequest::parse(args.clone()).unwrap_err();
    let error = editor_startup_argument_error(&args, source);

    assert_eq!(
        error.to_string(),
        "editor startup diagnostic: component=editor_app requested=--operation window.layout.reset --list-operations --headless cause=--operation, --list-operations, and --operation-history are mutually exclusive recovery=provide one valid editor startup mode and run zircon_editor --help to inspect supported arguments"
    );
}

#[test]
fn editor_startup_argument_summary_redacts_operation_payloads() {
    for (args, expected) in [
        (
            vec![
                "--operation".to_string(),
                "project.export".to_string(),
                "--args".to_string(),
                r#"{"token":"secret"}"#.to_string(),
                "--headless".to_string(),
            ],
            "--operation project.export --args <redacted> --headless",
        ),
        (
            vec![
                "--operation".to_string(),
                "project.export".to_string(),
                r#"--args={"token":"secret"}"#.to_string(),
                "--headless".to_string(),
            ],
            "--operation project.export --args=<redacted> --headless",
        ),
    ] {
        let summary = editor_startup_argument_summary(&args);

        assert_eq!(summary, expected);
        assert!(!summary.contains("secret"));
    }
}

#[test]
fn editor_startup_argument_error_redacts_payloads_from_requested_and_cause() {
    let secret = r#"{"token":"secret"}"#;
    for args in [
        vec!["--args".to_string(), secret.to_string()],
        vec![format!("--args={secret}")],
    ] {
        let source: Box<dyn std::error::Error> =
            format!("unknown editor argument `{}`", args.last().unwrap()).into();
        let diagnostic = editor_startup_argument_error(&args, source).to_string();

        assert!(!diagnostic.contains("secret"));
        assert!(diagnostic.contains("<redacted>"));
    }
}

#[test]
fn editor_cli_operation_startup_request_identifies_the_selected_mode() {
    for (args, expected) in [
        (
            vec![
                "--operation".to_string(),
                "window.layout.reset".to_string(),
                "--args".to_string(),
                r#"{"token":"secret"}"#.to_string(),
                "--headless".to_string(),
            ],
            "operation:window.layout.reset",
        ),
        (
            vec!["--list-operations".to_string(), "--headless".to_string()],
            "operation:list",
        ),
        (
            vec!["--operation-history".to_string(), "--headless".to_string()],
            "operation:history",
        ),
    ] {
        let request = EditorCliOperationRequest::parse(args).unwrap().unwrap();

        assert_eq!(request.startup_request(), expected);
        assert!(!request.startup_request().contains("secret"));
    }
}

#[test]
fn editor_cli_operation_startup_failure_reports_an_actionable_product_error() {
    let error = editor_operation_startup_error(
        "operation:window.layout.reset",
        "runtime session rejected the editor ABI",
    );

    assert_eq!(
        error.to_string(),
        "editor startup diagnostic: component=editor_operation requested=operation:window.layout.reset cause=editor operation startup failed: runtime session rejected the editor ABI recovery=verify the staged runtime ABI, editor operation registrations, and selected operation before retrying zircon_editor"
    );
}

#[test]
fn editor_cli_operation_finish_reports_teardown_after_success() {
    let error = finish_editor_operation(
        Ok::<_, Box<dyn std::error::Error>>(()),
        Some("destroy failed"),
    )
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "runtime session teardown failed: destroy failed"
    );
}

#[test]
fn editor_cli_operation_finish_preserves_primary_and_teardown_failures() {
    let error = finish_editor_operation::<(), _>(
        Err("gateway attach failed".into()),
        Some("destroy failed"),
    )
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor operation failed: gateway attach failed; runtime session teardown also failed: destroy failed"
    );
}

#[test]
fn editor_gui_finish_covers_host_and_teardown_outcomes() {
    assert!(finish_editor_host(
        "builtin_view:editor.scene",
        Ok::<_, Box<dyn std::error::Error>>(()),
        None::<&str>,
    )
    .is_ok());

    let host_error = finish_editor_host::<(), &str>(
        "builtin_view:editor.scene",
        Err("host failed".into()),
        None,
    )
    .unwrap_err();
    assert_eq!(host_error.to_string(), "host failed");

    let teardown_error = finish_editor_host(
        "builtin_view:editor.scene",
        Ok::<_, Box<dyn std::error::Error>>(()),
        Some("destroy failed"),
    )
    .unwrap_err();
    assert_eq!(
        teardown_error.to_string(),
        "editor startup diagnostic: component=runtime_session requested=builtin_view:editor.scene cause=runtime session teardown failed: destroy failed recovery=verify the runtime session lifecycle and staged runtime ABI before retrying zircon_editor"
    );

    let combined_error = finish_editor_host::<(), _>(
        "builtin_view:editor.scene",
        Err("host failed".into()),
        Some("destroy failed"),
    )
    .unwrap_err();
    assert_eq!(
        combined_error.to_string(),
        "editor startup diagnostic: component=editor_host requested=builtin_view:editor.scene cause=editor host failed: host failed; runtime session teardown also failed: destroy failed recovery=inspect both the editor host and runtime session failures before retrying zircon_editor"
    );
}

#[test]
fn editor_cli_operation_entry_wraps_post_parse_startup_failures() {
    let source = include_str!("../../editor.rs");
    let request_summary = source
        .find("let requested_operation = request.startup_request();")
        .expect("editor operation entry must summarize the parsed request");
    let operation_run = source[request_summary..]
        .find("Self::run_editor_operation(request)")
        .map(|offset| request_summary + offset)
        .expect("editor operation entry must execute the parsed request");
    let diagnostic_wrap = source[operation_run..]
        .find("editor_operation_startup_error(&requested_operation, error)")
        .map(|offset| operation_run + offset)
        .expect("editor operation entry must wrap post-parse startup failures");

    assert!(request_summary < operation_run);
    assert!(operation_run < diagnostic_wrap);
}

#[test]
fn editor_startup_diagnostics_have_one_private_module_owner() {
    let entry = include_str!("../../editor.rs");
    let diagnostics = include_str!("../startup_diagnostics.rs");

    assert!(entry.contains("mod startup_diagnostics;"));
    assert!(!entry.contains("struct EditorStartupDiagnosticError"));
    assert!(diagnostics.contains("pub(super) struct EditorStartupDiagnosticError"));
    assert!(diagnostics.contains("pub(super) fn finish_editor_host"));
    assert!(diagnostics.contains("pub(super) fn finish_editor_operation"));
}

#[test]
fn editor_cli_operation_parser_rejects_list_operations_mixed_with_history_query() {
    let error = EditorCliOperationRequest::parse([
        "--list-operations".to_string(),
        "--operation-history".to_string(),
        "--headless".to_string(),
    ])
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "--operation, --list-operations, and --operation-history are mutually exclusive"
    );
}

#[test]
fn editor_cli_operation_parser_rejects_control_request_without_headless() {
    for args in [
        vec!["--operation".to_string(), "window.layout.reset".to_string()],
        vec!["--list-operations".to_string()],
        vec!["--operation-history".to_string()],
    ] {
        let error = EditorCliOperationRequest::parse(args).unwrap_err();

        assert_eq!(
            error.to_string(),
            "editor operation control requests require --headless"
        );
    }
}

#[test]
fn editor_cli_operation_parser_rejects_duplicate_control_arguments() {
    for (args, expected) in [
        (
            vec![
                "--operation".to_string(),
                "window.layout.reset".to_string(),
                "--operation".to_string(),
                "window.layout.reset".to_string(),
                "--headless".to_string(),
            ],
            "--operation was provided more than once",
        ),
        (
            vec![
                "--operation".to_string(),
                "window.layout.reset".to_string(),
                "--args".to_string(),
                "{}".to_string(),
                "--args".to_string(),
                "{}".to_string(),
                "--headless".to_string(),
            ],
            "--args was provided more than once",
        ),
        (
            vec![
                "--operation".to_string(),
                "window.layout.reset".to_string(),
                "--operation-group".to_string(),
                "Group.1".to_string(),
                "--operation-group".to_string(),
                "Group.2".to_string(),
                "--headless".to_string(),
            ],
            "--operation-group was provided more than once",
        ),
        (
            vec![
                "--list-operations".to_string(),
                "--list-operations".to_string(),
                "--headless".to_string(),
            ],
            "--list-operations was provided more than once",
        ),
        (
            vec![
                "--operation-history".to_string(),
                "--operation-history".to_string(),
                "--headless".to_string(),
            ],
            "--operation-history was provided more than once",
        ),
        (
            vec![
                "--list-operations".to_string(),
                "--headless".to_string(),
                "--headless".to_string(),
            ],
            "--headless was provided more than once",
        ),
    ] {
        let error = EditorCliOperationRequest::parse(args).unwrap_err();

        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn editor_cli_operation_parser_leaves_empty_args_for_gui_startup() {
    assert!(EditorCliOperationRequest::parse(Vec::<String>::new())
        .unwrap()
        .is_none());
}

#[test]
fn editor_cli_operation_parser_accepts_list_operations() {
    let request = EditorCliOperationRequest::parse([
        "--list-operations".to_string(),
        "--headless".to_string(),
    ])
    .unwrap()
    .unwrap();

    assert!(request.list_operations);
    assert!(request.headless);
}

#[test]
fn editor_cli_operation_parser_accepts_operation_history_query() {
    let request = EditorCliOperationRequest::parse([
        "--operation-history".to_string(),
        "--headless".to_string(),
    ])
    .unwrap()
    .unwrap();

    assert!(request.query_operation_history);
    assert!(request.headless);
}

#[test]
fn editor_cli_operation_history_query_maps_to_control_request() {
    let request = EditorCliOperationRequest::parse([
        "--operation-history".to_string(),
        "--headless".to_string(),
    ])
    .unwrap()
    .unwrap();

    assert!(matches!(
        request.into_control_request().unwrap(),
        EditorOperationControlRequest::QueryOperationHistory
    ));
}
