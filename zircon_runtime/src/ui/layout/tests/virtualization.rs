use super::{
    fixed_extent_virtual_list_content_extent, fixed_extent_virtual_list_item_offset,
    fixed_extent_virtual_list_step_extent,
};

#[test]
fn fixed_extent_geometry_includes_non_negative_gap_without_iterating_items() {
    assert_eq!(fixed_extent_virtual_list_step_extent(24.0, 2.0), 26.0);
    assert_eq!(
        fixed_extent_virtual_list_content_extent(100_000, 24.0, 2.0),
        2_599_998.0
    );
    assert_eq!(
        fixed_extent_virtual_list_item_offset(50_000, 24.0, 2.0),
        1_300_000.0
    );
}
