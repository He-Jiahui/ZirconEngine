const VALUE_TEXT_OFFSET: i32 = 1;

pub(super) fn value_text_order(surface_order: i32) -> i32 {
    surface_order + VALUE_TEXT_OFFSET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_value_text_paints_above_field_surface() {
        let surface = 40;

        assert!(surface < value_text_order(surface));
    }
}
