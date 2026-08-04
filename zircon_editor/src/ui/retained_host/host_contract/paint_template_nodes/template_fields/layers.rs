pub(super) const SEARCH_GLYPH_OFFSET: i32 = 1;
pub(super) const SEARCH_CLEAR_ACTION_OFFSET: i32 = 2;
pub(super) const STEPPER_OFFSET: i32 = 2;
pub(super) const TEXT_OFFSET: i32 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_layer_offsets_keep_text_above_glyphs_and_stepper() {
        assert!(SEARCH_GLYPH_OFFSET < STEPPER_OFFSET);
        assert!(STEPPER_OFFSET < TEXT_OFFSET);
    }
}
