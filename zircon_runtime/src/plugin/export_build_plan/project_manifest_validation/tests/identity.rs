use super::validate_project_plugin_package_id;

#[test]
fn deferred_provider_diagnostic_context_preserves_contract() {
    let mut diagnostics = Vec::new();
    validate_project_plugin_package_id(
        format_args!("project plugin selection id"),
        "Bad__",
        &mut diagnostics,
    );
    validate_project_plugin_package_id(
        format_args!("project plugin feature streaming provider_package_id"),
        "Bad__",
        &mut diagnostics,
    );

    assert_eq!(
        diagnostics,
        vec![
            "project plugin selection id `Bad__` must start with a lowercase ASCII letter".to_string(),
            "project plugin selection id `Bad__` must contain only lowercase ASCII letters, digits, and underscores".to_string(),
            "project plugin selection id `Bad__` must not end with an underscore or contain repeated underscores".to_string(),
            "project plugin feature streaming provider_package_id `Bad__` must start with a lowercase ASCII letter".to_string(),
            "project plugin feature streaming provider_package_id `Bad__` must contain only lowercase ASCII letters, digits, and underscores".to_string(),
            "project plugin feature streaming provider_package_id `Bad__` must not end with an underscore or contain repeated underscores".to_string(),
        ]
    );
}

#[test]
fn project_feature_identity_validation_does_not_allocate_scan_helpers() {
    let source = include_str!("../identity.rs");
    let segment_collection = ["split('.')", ".collect::<Vec<_>>()"].concat();
    let formatted_prefix = ["format!(\"{owner_", "plugin_id}.\")"].concat();
    assert!(!source.contains(&segment_collection));
    assert!(!source.contains(&formatted_prefix));
}

#[test]
fn project_feature_owner_matching_preserves_the_dot_boundary() {
    assert!(super::project_feature_id_has_owner(
        "rendering",
        "rendering.deferred"
    ));
    assert!(!super::project_feature_id_has_owner(
        "render",
        "rendering.deferred"
    ));
}
