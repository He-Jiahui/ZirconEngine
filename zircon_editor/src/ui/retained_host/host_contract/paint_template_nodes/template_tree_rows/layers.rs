const INDENT_GUIDES_OFFSET: i32 = 1;
const DISCLOSURE_OFFSET: i32 = 2;
const OBJECT_ICON_OFFSET: i32 = 3;
const LABEL_OFFSET: i32 = 4;
const ACTION_SLOT_OFFSET: i32 = 5;
const PRIMARY_ACTION_ICON_OFFSET: i32 = 1;
const SECONDARY_ACTION_SLOT_OFFSET: i32 = 2;
const SECONDARY_ACTION_ICON_OFFSET: i32 = 3;

pub(super) fn indent_guides_order(surface_order: i32) -> i32 {
    surface_order + INDENT_GUIDES_OFFSET
}

pub(super) fn disclosure_order(surface_order: i32) -> i32 {
    surface_order + DISCLOSURE_OFFSET
}

pub(super) fn object_icon_order(surface_order: i32) -> i32 {
    surface_order + OBJECT_ICON_OFFSET
}

pub(super) fn label_order(surface_order: i32) -> i32 {
    surface_order + LABEL_OFFSET
}

pub(super) fn action_slot_order(surface_order: i32) -> i32 {
    surface_order + ACTION_SLOT_OFFSET
}

pub(super) fn primary_action_icon_order(action_order: i32) -> i32 {
    action_order + PRIMARY_ACTION_ICON_OFFSET
}

pub(super) fn secondary_action_slot_order(action_order: i32) -> i32 {
    action_order + SECONDARY_ACTION_SLOT_OFFSET
}

pub(super) fn secondary_action_icon_order(action_order: i32) -> i32 {
    action_order + SECONDARY_ACTION_ICON_OFFSET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_row_orders_keep_content_above_surface() {
        let surface = 20;
        let action_slot = action_slot_order(surface);

        assert!(surface < indent_guides_order(surface));
        assert!(indent_guides_order(surface) < disclosure_order(surface));
        assert!(disclosure_order(surface) < object_icon_order(surface));
        assert!(object_icon_order(surface) < label_order(surface));
        assert!(label_order(surface) < action_slot);
        assert!(action_slot < primary_action_icon_order(action_slot));
        assert!(primary_action_icon_order(action_slot) < secondary_action_slot_order(action_slot));
        assert!(
            secondary_action_slot_order(action_slot) < secondary_action_icon_order(action_slot)
        );
    }
}
