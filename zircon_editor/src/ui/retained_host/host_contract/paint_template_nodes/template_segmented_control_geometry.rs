use super::super::data::{FrameRect, TemplatePaneNodeData};

pub(super) const SEGMENT_FONT_SIZE: f32 = 11.0;
pub(super) const SEGMENT_TEXT_INSET_X: f32 = 8.0;
pub(super) const SEGMENT_TEXT_INSET_Y: f32 = 5.0;
pub(super) const SEGMENT_RADIUS: f32 = 5.0;
pub(super) const SEGMENT_GROUP_LABEL_FONT_SIZE: f32 = 11.0;
pub(super) const SEGMENT_GROUP_LABEL_HEIGHT: f32 = 14.0;
pub(super) const TAB_FONT_SIZE: f32 = 12.0;
pub(super) const TAB_UNDERLINE_HEIGHT: f32 = 2.0;

const SEGMENT_GROUP_LABEL_GAP: f32 = 4.0;
const SEGMENT_SELECTED_INSET: f32 = 2.0;
const TAB_TEXT_INSET_X: f32 = 12.0;

pub(super) fn segmented_group_label_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: SEGMENT_GROUP_LABEL_HEIGHT,
    }
}

pub(super) fn segmented_body_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let label_block_height = if node.label_text.trim().is_empty() {
        0.0
    } else {
        SEGMENT_GROUP_LABEL_HEIGHT + SEGMENT_GROUP_LABEL_GAP
    };

    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + label_block_height + node.layout_offset_y,
        width: rect.width,
        height: (rect.height - label_block_height).max(1.0),
    }
}

pub(super) fn tab_paint_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: rect.width,
        height: rect.height,
    }
}

pub(super) fn tab_underline_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y + (rect.height - TAB_UNDERLINE_HEIGHT).max(0.0),
        width: rect.width,
        height: TAB_UNDERLINE_HEIGHT,
    }
}

pub(super) fn tab_label_rect(rect: &FrameRect) -> FrameRect {
    let line_height = tab_line_height();
    FrameRect {
        x: rect.x + TAB_TEXT_INSET_X,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.width - TAB_TEXT_INSET_X * 2.0).max(1.0),
        height: line_height,
    }
}

pub(super) fn segment_divider_rect(segment: &FrameRect) -> FrameRect {
    FrameRect {
        x: segment.x,
        y: segment.y + 4.0,
        width: 1.0,
        height: (segment.height - 8.0).max(1.0),
    }
}

pub(super) fn selected_segment_rect(segment: &FrameRect) -> FrameRect {
    inset_rect(segment, SEGMENT_SELECTED_INSET)
}

pub(super) fn selected_segment_underline_rect(
    selected_rect: &FrameRect,
    underline_height: f32,
) -> FrameRect {
    FrameRect {
        x: selected_rect.x,
        y: selected_rect.y + (selected_rect.height - underline_height).max(0.0),
        width: selected_rect.width,
        height: underline_height.min(selected_rect.height).max(1.0),
    }
}

pub(super) fn segment_label_rect(segment: &FrameRect) -> FrameRect {
    FrameRect {
        x: segment.x + SEGMENT_TEXT_INSET_X,
        y: segment.y + SEGMENT_TEXT_INSET_Y,
        width: (segment.width - SEGMENT_TEXT_INSET_X * 2.0).max(1.0),
        height: (segment.height - SEGMENT_TEXT_INSET_Y * 2.0).max(1.0),
    }
}

pub(super) fn segment_rect(rect: &FrameRect, index: usize, count: usize) -> FrameRect {
    let count = count.max(1);
    let width = rect.width / count as f32;
    FrameRect {
        x: rect.x + width * index as f32,
        y: rect.y,
        width: if index + 1 == count {
            rect.x + rect.width - (rect.x + width * index as f32)
        } else {
            width
        }
        .max(1.0),
        height: rect.height,
    }
}

pub(super) fn segment_line_height() -> f32 {
    SEGMENT_FONT_SIZE * 1.2
}

pub(super) fn segment_group_label_line_height() -> f32 {
    SEGMENT_GROUP_LABEL_FONT_SIZE * 1.2
}

pub(super) fn tab_line_height() -> f32 {
    TAB_FONT_SIZE * 1.2
}

fn inset_rect(rect: &FrameRect, inset: f32) -> FrameRect {
    FrameRect {
        x: rect.x + inset,
        y: rect.y + inset,
        width: (rect.width - inset * 2.0).max(1.0),
        height: (rect.height - inset * 2.0).max(1.0),
    }
}
