use super::*;

#[test]
fn field_layer_offsets_keep_text_above_glyphs_and_stepper() {
    assert!(SEARCH_GLYPH_OFFSET < STEPPER_OFFSET);
    assert!(STEPPER_OFFSET < TEXT_OFFSET);
}
