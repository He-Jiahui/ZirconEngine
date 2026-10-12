use super::*;

#[test]
fn icon_button_glyph_order_stays_above_surface() {
    let surface = 10;
    let glyph = glyph_order(surface);

    assert_eq!(glyph, 12);
    assert!(surface < glyph);
}
