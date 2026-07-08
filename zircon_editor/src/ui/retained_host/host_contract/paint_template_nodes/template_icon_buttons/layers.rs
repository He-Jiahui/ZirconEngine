const GLYPH_OFFSET: i32 = 2;

pub(super) fn glyph_order(surface_order: i32) -> i32 {
    surface_order + GLYPH_OFFSET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_button_glyph_order_stays_above_surface() {
        let surface = 10;
        let glyph = glyph_order(surface);

        assert_eq!(glyph, 12);
        assert!(surface < glyph);
    }
}
