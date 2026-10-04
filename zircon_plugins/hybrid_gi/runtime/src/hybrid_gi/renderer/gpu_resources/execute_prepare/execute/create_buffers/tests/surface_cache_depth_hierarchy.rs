use super::*;

#[test]
fn surface_cache_depth_hierarchy_stops_at_one_texel_per_aligned_page() {
    assert_eq!(surface_cache_depth_hierarchy_mip_level_count((512, 64)), 7);
    assert_eq!(surface_cache_depth_hierarchy_mip_level_count((512, 128)), 7);
    assert_eq!(surface_cache_depth_hierarchy_mip_level_count((1, 1)), 1);
}
