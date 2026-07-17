const SURFACE_OVERLAY_OFFSET: i32 = 1;
const CONTENT_OFFSET: i32 = 2;
const LABEL_OFFSET: i32 = 1;

pub(super) fn surface_overlay_order(surface_order: i32) -> i32 {
    surface_order + SURFACE_OVERLAY_OFFSET
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
    fn button_orders_keep_surface_overlay_content_and_label_stack() {
        let surface = 12;
        let overlay = surface_overlay_order(surface);
        let content = content_order(surface);
        let label = label_order(content);

        assert_eq!(overlay, 13);
        assert_eq!(content, 14);
        assert_eq!(label, 15);
        assert!(surface < overlay);
        assert!(overlay < content);
        assert!(content < label);
    }
}
