use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

use super::super::identity::alert_has_action;
use super::metrics::{ALERT_ACTION_EDGE, ALERT_ACTION_GAP, ALERT_ACTION_TRAILING};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_action_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    if alert_has_action(node) {
        ALERT_ACTION_EDGE + ALERT_ACTION_GAP
    } else {
        0.0
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_action_frame(
    rect: &FrameRect,
) -> FrameRect {
    let edge = ALERT_ACTION_EDGE.min(rect.height - 8.0).max(1.0);
    FrameRect {
        x: rect.x + rect.width - ALERT_ACTION_TRAILING - edge,
        y: rect.y + (rect.height - edge).max(0.0) * 0.5,
        width: edge,
        height: edge,
    }
}
