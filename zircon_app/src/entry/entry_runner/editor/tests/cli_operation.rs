use super::super::{
    editor_startup_argument_error, editor_startup_argument_summary, EditorCliOperationRequest,
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
    let summary = editor_startup_argument_summary(&[
        "--operation".to_string(),
        "project.export".to_string(),
        "--args".to_string(),
        r#"{"token":"secret"}"#.to_string(),
        "--headless".to_string(),
    ]);

    assert_eq!(
        summary,
        "--operation project.export --args <redacted> --headless"
    );
    assert!(!summary.contains("secret"));
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
