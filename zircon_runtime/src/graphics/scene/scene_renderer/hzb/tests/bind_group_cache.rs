use super::{HzbOcclusionBindGroupKey, MAX_HZB_OCCLUSION_BIND_GROUPS};
use crate::graphics::scene::scene_renderer::hzb::HzbSampledResourceIdentity;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshIndirectResourceIdentity;

#[test]
fn hzb_bind_group_cache_is_bounded() {
    assert_eq!(MAX_HZB_OCCLUSION_BIND_GROUPS, 64);
}

#[test]
fn hzb_bind_group_key_tracks_sampled_texture_and_indirect_resource_revision() {
    let sampled_a = HzbSampledResourceIdentity::new();
    let sampled_b = HzbSampledResourceIdentity::new();
    let indirect_a = MeshIndirectResourceIdentity::new(7, 1);
    let indirect_b = MeshIndirectResourceIdentity::new(7, 2);

    let key = HzbOcclusionBindGroupKey::new(sampled_a, indirect_a);

    assert_ne!(key, HzbOcclusionBindGroupKey::new(sampled_b, indirect_a));
    assert_ne!(key, HzbOcclusionBindGroupKey::new(sampled_a, indirect_b));
    assert_eq!(key, HzbOcclusionBindGroupKey::new(sampled_a, indirect_a));
}
