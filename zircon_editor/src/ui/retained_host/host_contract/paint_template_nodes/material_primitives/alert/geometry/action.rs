use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

use super::super::identity::alert_has_action;
use super::metrics::{
    alert_bounded_extent, ALERT_ACTION_EDGE, ALERT_ACTION_GAP, ALERT_ACTION_TRAILING,
};

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
    let width = alert_bounded_extent(rect.width);
    let height = alert_bounded_extent(rect.height);
    let edge = ALERT_ACTION_EDGE
        .min(width)
        .min((height - ALERT_ACTION_TRAILING).max(0.0));
    let trailing = ALERT_ACTION_TRAILING.min((width - edge).max(0.0));
    FrameRect {
        x: rect.x + width - trailing - edge,
        y: rect.y + (height - edge) * 0.5,
        width: edge,
        height: edge,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_action_stays_inside_tight_alert_bounds() {
        let alert = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 0.4,
            height: 0.6,
        };
        let frame = alert_action_frame(&alert);

        assert!(frame.x >= alert.x);
        assert!(frame.y >= alert.y);
        assert!(frame.right() <= alert.right());
        assert!(frame.bottom() <= alert.bottom());
    }
}
