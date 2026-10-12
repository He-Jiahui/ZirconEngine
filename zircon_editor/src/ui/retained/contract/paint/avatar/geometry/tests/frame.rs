use super::*;

#[test]
fn avatar_frame_stays_inside_tight_parent_bounds() {
    let parent = FrameRect {
        x: 10.4,
        y: 20.8,
        width: 0.4,
        height: 0.6,
    };
    let frame = avatar_frame(&parent);

    assert!(frame.x >= parent.x);
    assert!(frame.y >= parent.y);
    assert!(frame.right() <= parent.right());
    assert!(frame.bottom() <= parent.bottom());
}
