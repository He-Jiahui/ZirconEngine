use super::super::data::{FrameRect, TemplatePaneNodeData};

pub(super) const TREE_FONT_SIZE: f32 = 12.0;
pub(super) const TREE_ROW_RADIUS: f32 = 5.0;
pub(super) const TREE_GUIDE_COLOR: [u8; 4] = [42, 55, 64, 255];
const TREE_BASE_INSET_X: f32 = 12.0;
const TREE_DISCLOSURE_SIZE: f32 = 12.0;
const TREE_ICON_SIZE: f32 = 14.0;
const TREE_TEXT_GAP: f32 = 7.0;
const TREE_RIGHT_INSET: f32 = 12.0;
const TREE_ACTION_SIZE: f32 = 14.0;
const TREE_ACTION_GAP: f32 = 16.0;
const TREE_GUIDE_STEP: f32 = 18.0;

pub(super) fn tree_disclosure_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let indent = if node.tree_indent_px.is_finite() && node.tree_indent_px > 0.0 {
        node.tree_indent_px
    } else {
        node.tree_depth.max(0) as f32 * TREE_GUIDE_STEP
    };
    FrameRect {
        x: rect.x + TREE_BASE_INSET_X + indent,
        y: rect.y + (rect.height - TREE_DISCLOSURE_SIZE).max(0.0) * 0.5,
        width: TREE_DISCLOSURE_SIZE,
        height: TREE_DISCLOSURE_SIZE,
    }
}

pub(super) fn tree_icon_rect(disclosure: &FrameRect) -> FrameRect {
    FrameRect {
        x: disclosure.x + disclosure.width + 4.0,
        y: disclosure.y + (disclosure.height - TREE_ICON_SIZE).max(0.0) * 0.5,
        width: TREE_ICON_SIZE,
        height: TREE_ICON_SIZE,
    }
}

pub(super) fn tree_action_rect(rect: &FrameRect, index_from_right: usize) -> FrameRect {
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

pub(super) fn tree_guide_x(rect: &FrameRect, level: usize) -> f32 {
    rect.x + TREE_BASE_INSET_X + 5.0 + level as f32 * TREE_GUIDE_STEP
}

pub(super) fn tree_label_rect(rect: &FrameRect, icon: &FrameRect) -> FrameRect {
    let line_height = tree_line_height();
    let text_x = icon.x + icon.width + TREE_TEXT_GAP;
    let right_reserve = TREE_RIGHT_INSET + TREE_ACTION_SIZE * 2.0 + TREE_ACTION_GAP;
    FrameRect {
        x: text_x,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.x + rect.width - text_x - right_reserve).max(1.0),
        height: line_height,
    }
}

pub(super) fn tree_line_height() -> f32 {
    TREE_FONT_SIZE * 1.2
}
