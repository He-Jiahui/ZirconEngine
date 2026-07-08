use super::*;

#[test]
fn native_bitmap_atlas_frame_index_advances_monotonically() {
    let mut frame_index = 0;

    assert_eq!(next_native_bitmap_atlas_frame_index(&mut frame_index), 1);
    assert_eq!(next_native_bitmap_atlas_frame_index(&mut frame_index), 2);

    frame_index = u64::MAX;
    assert_eq!(
        next_native_bitmap_atlas_frame_index(&mut frame_index),
        u64::MAX
    );
}
