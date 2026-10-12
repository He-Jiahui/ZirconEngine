use super::{coalesced_vertex_ranges, sdf_vertex_buffer_capacity};

#[test]
fn sdf_vertex_buffer_capacity_grows_by_power_of_two_and_never_shrinks_requirement() {
    assert_eq!(sdf_vertex_buffer_capacity(0), 0);
    assert_eq!(sdf_vertex_buffer_capacity(1), 4 * 1024);
    assert_eq!(sdf_vertex_buffer_capacity(4 * 1024), 4 * 1024);
    assert_eq!(sdf_vertex_buffer_capacity(4 * 1024 + 1), 8 * 1024);
}

#[test]
fn changed_vertex_ranges_are_bounded_sorted_and_coalesced() {
    assert_eq!(
        coalesced_vertex_ranges(&[8..12, 2..4, 4..7, 20..24, 7..8], 16),
        vec![2..12]
    );
}
