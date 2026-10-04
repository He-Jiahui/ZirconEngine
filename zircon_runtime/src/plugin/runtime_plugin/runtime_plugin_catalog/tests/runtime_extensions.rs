#[test]
fn project_extension_report_does_not_complete_an_already_completed_manifest() {
    let source = include_str!("../runtime_extensions.rs");
    let repeated_completion = ["self", ".feature_dependency_report(&completed"].concat();
    assert!(!source.contains(&repeated_completion));
}
