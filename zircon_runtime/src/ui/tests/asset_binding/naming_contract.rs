#[test]
fn binding_validation_source_boundary_uses_schema_name_not_milestone() {
    let source = include_str!("../../template/asset/binding/validation.rs");

    assert!(source.contains("fn is_runtime_binding_expression("));
    assert!(!source.to_ascii_lowercase().contains("m18"));
}
