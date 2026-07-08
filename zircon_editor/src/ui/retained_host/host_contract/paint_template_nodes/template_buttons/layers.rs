const SURFACE_INDICATOR_OFFSET: i32 = 1;
const CONTENT_OFFSET: i32 = 2;
const LABEL_OFFSET: i32 = 1;

pub(super) fn surface_indicator_order(surface_order: i32) -> i32 {
    surface_order + SURFACE_INDICATOR_OFFSET
}

pub(super) fn content_order(surface_order: i32) -> i32 {
    surface_order + CONTENT_OFFSET
}

pub(super) fn label_order(content_order: i32) -> i32 {
    content_order + LABEL_OFFSET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_orders_keep_surface_indicator_content_and_label_stack() {
        let surface = 12;
        let indicator = surface_indicator_order(surface);
        let content = content_order(surface);
        let label = label_order(content);

        assert_eq!(indicator, 13);
        assert_eq!(content, 14);
        assert_eq!(label, 15);
        assert!(surface < indicator);
        assert!(indicator < content);
        assert!(content < label);
    }
}
