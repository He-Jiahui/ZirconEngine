use super::*;

#[test]
fn chip_delete_slot_stays_inside_tight_chip_bounds() {
    let chip = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 0.4,
        height: 0.6,
    };
    let frame = chip_delete_icon_frame(&TemplatePaneNodeData::default(), &chip);

    assert!(frame.x >= chip.x);
    assert!(frame.y >= chip.y);
    assert!(frame.right() <= chip.right());
    assert!(frame.bottom() <= chip.bottom());
}
