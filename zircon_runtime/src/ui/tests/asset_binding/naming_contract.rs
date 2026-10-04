//! 守住绑定验证器的来源命名边界，避免产品契约重新依赖历史里程碑代号。
#[test]
fn binding_validation_source_boundary_uses_schema_name_not_milestone() {
    let source = include_str!("../../template/asset/binding/validation.rs");

    assert!(source.contains("fn is_runtime_binding_expression("));
    assert!(!source.to_ascii_lowercase().contains("m18"));
}
