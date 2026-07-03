use super::super::super::data::FrameRect;
use super::metrics::{
    TREE_ACTION_BUTTON_SIZE, TREE_ACTION_GAP, TREE_ACTION_SIZE, TREE_RIGHT_INSET,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_action_rect(
    rect: &FrameRect,
    index_from_right: usize,
) -> FrameRect {
    let stride = TREE_ACTION_SIZE + TREE_ACTION_GAP;
    FrameRect {
        x: rect.x + rect.width
            - TREE_RIGHT_INSET
            - TREE_ACTION_SIZE
            - index_from_right as f32 * stride,
        y: rect.y + (rect.height - TREE_ACTION_SIZE).max(0.0) * 0.5,
        width: TREE_ACTION_SIZE,
        height: TREE_ACTION_SIZE,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_action_button_rect(
    rect: &FrameRect,
    index_from_right: usize,
) -> FrameRect {
    let icon = tree_action_rect(rect, index_from_right);
    FrameRect {
        x: icon.x + (icon.width - TREE_ACTION_BUTTON_SIZE) * 0.5,
        y: icon.y + (icon.height - TREE_ACTION_BUTTON_SIZE) * 0.5,
        width: TREE_ACTION_BUTTON_SIZE,
        height: TREE_ACTION_BUTTON_SIZE,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_action_icon_rect(
    button_rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: button_rect.x + (button_rect.width - TREE_ACTION_SIZE).max(0.0) * 0.5,
        y: button_rect.y + (button_rect.height - TREE_ACTION_SIZE).max(0.0) * 0.5,
        width: TREE_ACTION_SIZE,
        height: TREE_ACTION_SIZE,
    }
}
