#[test]
fn geometry_indexes_descriptor_rows_by_borrowed_id() {
    let source = include_str!("../compute.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    assert!(!implementation.contains("descriptor.descriptor_id.clone()"));
    assert!(implementation.contains("HashMap<&str, &ViewDescriptor>"));
}
