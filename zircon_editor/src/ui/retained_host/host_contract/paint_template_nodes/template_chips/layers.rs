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
    fn chip_orders_keep_label_below_chevron() {
        let surface = 8;
        let label = label_order(surface);
        let chevron = chevron_order(surface);

        assert_eq!(label, 10);
        assert_eq!(chevron, 11);
        assert!(surface < label);
        assert!(label < chevron);
    }
}
