use zircon_runtime::core::framework::render::{
    IblBakeArtifactContents, IblBakeArtifactDescriptor, IblBakeArtifactRequest, ProceduralSkyParams,
};

#[test]
fn runtime_environment_ibl_artifact_rejects_different_source_layout() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);
    let matching = IblBakeArtifactRequest::new(key, 128, 8);
    let different_source = IblBakeArtifactRequest::new(key, 256, 9);

    assert!(descriptor.is_current_for(&matching));
    assert!(!descriptor.is_current_for(&different_source));

    let rebuilt = IblBakeArtifactDescriptor::current_for_request(&different_source);
    assert_eq!(rebuilt.source_face_size(), 256);
    assert_eq!(rebuilt.source_mip_count(), 9);
    assert!(rebuilt.is_current_for(&different_source));
}
