use super::super::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::popup_anchor_metrics::SLATE_POPUP_ANCHOR_METRICS;

pub(in crate::ui::retained_host::host_contract) const TEMPLATE_POPUP_ANCHOR_GAP: f32 =
    SLATE_POPUP_ANCHOR_METRICS.anchor_gap;
const MIN_TEMPLATE_POPUP_ROW_HEIGHT: f32 = 24.0;

pub(in crate::ui::retained_host::host_contract) fn dropdown_option_row_height(
    control_frame: &FrameRect,
) -> f32 {
    control_frame.height.max(MIN_TEMPLATE_POPUP_ROW_HEIGHT)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PopupRowLayout {
    pub(super) left: f32,
    pub(super) right: f32,
    pub(super) top: f32,
    pub(super) bottom: f32,
    pub(super) spacing: f32,
}

pub(super) fn popup_row_layout(node: &TemplatePaneNodeData) -> PopupRowLayout {
    PopupRowLayout {
        left: nonnegative_metric(node.layout_padding_left),
        right: nonnegative_metric(node.layout_padding_right),
        top: nonnegative_metric(node.layout_padding_top),
        bottom: nonnegative_metric(node.layout_padding_bottom),
        spacing: nonnegative_metric(node.layout_spacing),
    }
}

pub(super) fn popup_row_height(
    node: &TemplatePaneNodeData,
    frame: &FrameRect,
    row_count: usize,
) -> Option<f32> {
    if row_count == 0 || !frame.height.is_finite() || frame.height <= 0.0 {
        return None;
    }
    let layout = popup_row_layout(node);
    let fixed_height =
        layout.top + layout.bottom + layout.spacing * row_count.saturating_sub(1) as f32;
    let height = (frame.height - fixed_height) / row_count as f32;
    (height.is_finite() && height > 0.0).then_some(height)
}

pub(super) fn popup_rows_height(
    node: &TemplatePaneNodeData,
    row_count: usize,
    row_height: f32,
) -> Option<f32> {
    if row_count == 0 || !row_height.is_finite() || row_height <= 0.0 {
        return None;
    }
    let layout = popup_row_layout(node);
    Some(
        layout.top
            + layout.bottom
            + row_height * row_count as f32
            + layout.spacing * row_count.saturating_sub(1) as f32,
    )
}

fn nonnegative_metric(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
