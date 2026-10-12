use super::*;

#[test]
fn text_field_alignment_stays_inside_fractional_declared_bounds() {
    let declared = FrameRect {
        x: 12.3,
        y: 8.4,
        width: 95.2,
        height: 30.5,
    };

    let aligned = paint_rect(&declared);

    assert_eq!(aligned.x, declared.x);
    assert_eq!(aligned.y, declared.y);
    assert_eq!(aligned.width, declared.width);
    assert_eq!(aligned.height, declared.height);
}
