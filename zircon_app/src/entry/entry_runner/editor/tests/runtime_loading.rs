#[test]
fn product_editor_paths_load_the_staged_runtime_library() {
    let source = include_str!("../../editor.rs");

    assert_eq!(source.matches("LoadedRuntime::load_default()").count(), 2);
    assert!(
        !source.contains("LoadedRuntime::linked()?"),
        "product editor entry paths must not bypass the staged runtime library"
    );
    assert!(
        !source.contains("RuntimeSession::create_linked_with_profile_and_project("),
        "product editor entry paths must create sessions through the staged runtime ABI"
    );
    assert!(!source.contains("RuntimeSession::create_with_profile_and_project("));
    assert!(
        source.contains("RuntimeSession::create_with_profile(runtime, b\"editor\")"),
        "the GUI gateway session must stay projectless because EditorUiHost owns project activation"
    );

    for component in [
        "editor_project",
        "editor_bootstrap",
        "editor_manager",
        "runtime_library",
        "runtime_session",
        "editor_gateway",
    ] {
        assert!(
            source.contains(&format!("\"{component}\"")),
            "GUI startup failures must identify the {component} component"
        );
    }
}

#[test]
fn product_editor_collects_runtime_session_teardown_before_returning_success() {
    let source = include_str!("../../editor.rs");
    let mut offset = 0;
    for needle in [
        "let runtime_teardown_failure = runtime_session.teardown_failure_state();",
        "let host_result: Result<_, Box<dyn Error>> = (|| {",
        "let runtime_gateway = runtime_session",
        "let host_config = editor_host_run_config_with_first_frame_exit(",
        "run_editor_with_config(core, runtime_gateway, host_config)",
        "Ok(())",
        "drop(runtime_session);",
        "finish_editor_host(",
        "host_result",
        "runtime_teardown_failure.take()",
        ")?;",
        "Ok(0)",
    ] {
        let index = source[offset..]
            .find(needle)
            .unwrap_or_else(|| panic!("editor runtime teardown path is missing `{needle}`"));
        offset += index + needle.len();
    }
}

#[test]
fn product_editor_operation_collects_runtime_session_teardown_before_returning_response() {
    let source = include_str!("../../editor.rs");
    let operation = source
        .split("fn run_editor_operation")
        .nth(1)
        .expect("editor CLI operation path should exist");
    let mut offset = 0;
    for needle in [
        "let runtime_teardown_failure = runtime_session.teardown_failure_state();",
        "let operation_result: Result<_, Box<dyn Error>> = (|| {",
        "runtime.attach_play_gateway(runtime_gateway)?;",
        "let response = runtime.handle_operation_control_request_from_source(",
        "Ok(response)",
        "drop(runtime);",
        "drop(runtime_session);",
        "finish_editor_operation(operation_result, runtime_teardown_failure.take())",
    ] {
        let index = operation[offset..]
            .find(needle)
            .unwrap_or_else(|| panic!("editor CLI runtime teardown path is missing `{needle}`"));
        offset += index + needle.len();
    }
}
