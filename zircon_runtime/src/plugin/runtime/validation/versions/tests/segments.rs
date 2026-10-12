#[test]
fn package_semver_validation_does_not_collect_segments() {
    let source = include_str!("../segments.rs");
    let allocating_shape = ["split('.')", ".collect::<Vec<_>>()"].concat();
    assert!(!source.contains(&allocating_shape));
}

#[test]
fn package_semver_validation_preserves_shape_and_component_diagnostics() {
    let mut diagnostics = Vec::new();
    super::validate_runtime_plugin_package_semver_segments("version", "1.2.3", &mut diagnostics);
    assert!(diagnostics.is_empty());

    super::validate_runtime_plugin_package_semver_segments("version", "1.2", &mut diagnostics);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].contains("must use MAJOR.MINOR.PATCH form"));

    diagnostics.clear();
    super::validate_runtime_plugin_package_semver_segments("version", "1..3", &mut diagnostics);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].contains("minor component `` must contain ASCII digits"));
}
