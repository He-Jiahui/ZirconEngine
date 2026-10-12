use super::VirtualGeometryRuntimeExtractOutput;

#[test]
fn runtime_extract_output_moves_all_parts_without_clone_projection() {
    let (extract, cpu_references, bvh_instances, resident_payloads) =
        VirtualGeometryRuntimeExtractOutput::default().into_parts();

    assert_eq!(extract, Default::default());
    assert!(cpu_references.is_empty());
    assert!(bvh_instances.is_empty());
    assert!(resident_payloads.is_empty());
}
