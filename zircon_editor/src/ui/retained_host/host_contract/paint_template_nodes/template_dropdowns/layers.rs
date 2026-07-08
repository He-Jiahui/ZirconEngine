const LABEL_OFFSET: i32 = 2;
const CHEVRON_OFFSET: i32 = 3;

pub(super) fn label_order(surface_order: i32) -> i32 {
    surface_order + LABEL_OFFSET
}

pub(super) fn chevron_order(surface_order: i32) -> i32 {
    surface_order + CHEVRON_OFFSET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropdown_layers_keep_surface_label_chevron_order() {
        let surface = 40;

        assert!(surface < label_order(surface));
        assert!(label_order(surface) < chevron_order(surface));
    }
}
