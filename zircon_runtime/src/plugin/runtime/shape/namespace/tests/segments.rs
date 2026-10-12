#[test]
fn runtime_feature_namespace_validation_streams_segments() {
    let source = include_str!("../../../feature_validation/shape/namespace/segments.rs");
    let allocating_shape = ["split('.')", ".collect::<Vec<_>>()"].concat();
    assert!(!source.contains(&allocating_shape));
}
