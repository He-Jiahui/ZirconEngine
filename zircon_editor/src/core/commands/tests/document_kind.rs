#[test]
fn document_kind_validation_streams_segments() {
    let source = include_str!("../document_kind.rs");
    let collecting_shape = ["split('.')", ".collect::<Vec<_>>()"].concat();
    assert!(!source.contains(&collecting_shape));
}
