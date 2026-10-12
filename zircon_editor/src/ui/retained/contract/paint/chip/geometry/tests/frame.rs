use super::*;

#[test]
fn chip_frame_stays_inside_tight_parent_bounds() {
    let parent = FrameRect {
        x: 10.4,
        y: 20.8,
        width: 0.4,
        height: 0.6,
    };
    let frame = chip_frame(&TemplatePaneNodeData::default(), &parent);

    assert!(frame.x >= parent.x);
    assert!(frame.y >= parent.y);
    assert!(frame.right() <= parent.right());
    assert!(frame.bottom() <= parent.bottom());
}

#[test]
fn chip_corner_radius_does_not_exceed_narrow_frame_bounds() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 2.0,
        height: 20.0,
    };

    assert_eq!(
        chip_corner_radius(&TemplatePaneNodeData::default(), &rect),
        1.0
    );
}
