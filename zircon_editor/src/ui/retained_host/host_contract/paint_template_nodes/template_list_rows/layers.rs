const SELECTION_INDICATOR_OFFSET: i32 = 1;
const LABEL_OFFSET: i32 = 2;
const ADORNMENT_OFFSET: i32 = 3;

pub(super) fn selection_indicator_order(surface_order: i32) -> i32 {
    surface_order + SELECTION_INDICATOR_OFFSET
}

pub(super) fn label_order(surface_order: i32) -> i32 {
    surface_order + LABEL_OFFSET
}

pub(super) fn adornment_order(surface_order: i32) -> i32 {
    surface_order + ADORNMENT_OFFSET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_row_layers_keep_surface_indicator_label_adornment_order() {
        let surface = 12;

        assert!(surface < selection_indicator_order(surface));
        assert!(selection_indicator_order(surface) < label_order(surface));
        assert!(label_order(surface) < adornment_order(surface));
    }
}
