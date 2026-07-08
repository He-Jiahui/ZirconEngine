const MARK_CONTENT_ORDER_OFFSET: i32 = 1;
const LABEL_ORDER_OFFSET: i32 = 2;
const TOGGLE_LABEL_ORDER_OFFSET: i32 = 1;
const TOGGLE_THUMB_ORDER_OFFSET: i32 = 2;

pub(super) fn mark_content_order(mark_order: i32) -> i32 {
    mark_order + MARK_CONTENT_ORDER_OFFSET
}

pub(super) fn mark_label_order(mark_order: i32) -> i32 {
    mark_order + LABEL_ORDER_OFFSET
}

pub(super) fn toggle_label_order(track_order: i32) -> i32 {
    track_order + TOGGLE_LABEL_ORDER_OFFSET
}

pub(super) fn toggle_thumb_order(track_order: i32) -> i32 {
    track_order + TOGGLE_THUMB_ORDER_OFFSET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_control_layers_keep_surface_content_label_thumb_order() {
        let order = 40;

        assert!(order < mark_content_order(order));
        assert!(mark_content_order(order) < mark_label_order(order));
        assert!(order < toggle_label_order(order));
        assert!(toggle_label_order(order) < toggle_thumb_order(order));
    }
}
