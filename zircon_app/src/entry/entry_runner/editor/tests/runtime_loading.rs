#[test]
fn product_editor_paths_load_the_staged_runtime_library() {
    let source = include_str!("../../editor.rs");

    assert_eq!(source.matches("LoadedRuntime::load_default()").count(), 0);
    let runtime_preflight = source
        .find("LoadedRuntime::preflight_default()")
        .expect("project startup must preflight the selected runtime BuildSet");
    let project_prepare = source
        .find("prepare_editor_gui_startup(gui_startup_request)")
        .expect("project startup must prepare its project session");
    assert!(
        runtime_preflight < project_prepare,
        "the runtime BuildSet must be checked before project materialization"
    );
    assert!(
        source.contains("runtime_preflight.load_after_preflight()"),
        "every GUI path must retain and revalidate the preflight used by embedded Play"
    );
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
    let manager_resolution = source
        .find(".resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)")
        .expect("the editor manager must be resolved during startup validation");
    let manager_release = source
        .find("drop(editor_manager);")
        .expect("the editor manager validation reference must be released explicitly");
    let runtime_load = source
        .find("runtime_preflight.load_after_preflight()")
        .expect("the staged runtime library must be loaded");
    assert!(
        manager_resolution < manager_release && manager_release < runtime_load,
        "the editor manager validation reference must not outlive product composition teardown"
    );

    for component in [
        "editor_project",
        "runtime_build_set",
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
        "let product_failure_ledger = runtime_teardown_failure.failure_ledger();",
        "let host_result: Result<_, Box<dyn Error>> = (|| {",
        "let runtime_gateway = runtime_session",
        "let host_config = editor_host_run_config_with_first_frame_exit(",
        "run_editor_with_config(core, runtime_gateway, host_config)",
        "Ok(())",
        "record_editor_host_failure(&product_failure_ledger, &host_result);",
        "drop(product_composition);",
        "drop(runtime_session);",
        "let failure_report = product_failure_ledger.snapshot();",
        "finish_editor_host(",
        "host_result",
        "failure_report",
        ")?;",
        "Ok(0)",
    ] {
        let index = source[offset..]
            .find(needle)
            .unwrap_or_else(|| panic!("editor runtime teardown path is missing `{needle}`"));
        offset += index + needle.len();
    }
}
