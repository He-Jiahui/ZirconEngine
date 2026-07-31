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
