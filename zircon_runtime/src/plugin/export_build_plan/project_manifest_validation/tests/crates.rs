use super::{validate_project_runtime_crate_name, validate_project_selection_runtime_crate_name};

#[test]
fn deferred_crate_diagnostic_context_preserves_contract() {
    let mut diagnostics = Vec::new();
    validate_project_selection_runtime_crate_name(
        format_args!("project plugin audio runtime_crate"),
        "Zircon__",
        &mut diagnostics,
    );
    validate_project_runtime_crate_name(
        format_args!("project plugin audio editor_crate"),
        "Zircon__",
        &mut diagnostics,
    );

    assert_eq!(
        diagnostics,
        vec![
            "project plugin audio runtime_crate `Zircon__` must use `zircon_plugin_` crate prefix or `builtin_` runtime-domain prefix and contain only lowercase ASCII letters, digits, and underscores".to_string(),
            "project plugin audio runtime_crate `Zircon__` must not end with an underscore or contain repeated underscores".to_string(),
            "project plugin audio editor_crate `Zircon__` must use `zircon_plugin_` prefix and contain only lowercase ASCII letters, digits, and underscores".to_string(),
            "project plugin audio editor_crate `Zircon__` must not end with an underscore or contain repeated underscores".to_string(),
        ]
    );
}
