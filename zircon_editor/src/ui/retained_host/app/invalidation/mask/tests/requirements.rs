use super::HostInvalidationMask;

#[test]
fn workbench_projection_requires_a_host_commit_without_promoting_global_presentation() {
    let mask = HostInvalidationMask::WORKBENCH_PROJECTION;

    assert!(mask.requires_host_recompute());
    assert!(!mask.requires_layout());
    assert!(!mask.requires_presentation());
    assert!(!mask.requires_hit_test());
}
