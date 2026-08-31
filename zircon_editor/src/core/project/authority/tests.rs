#[test]
fn canonical_project_resolution_does_not_repeat_link_component_validation() {
    let source = include_str!("open_project.rs");
    let repeated_validation = ["validate_existing_project_root", "(&root)"].concat();

    assert!(!source.contains(&repeated_validation));
}
