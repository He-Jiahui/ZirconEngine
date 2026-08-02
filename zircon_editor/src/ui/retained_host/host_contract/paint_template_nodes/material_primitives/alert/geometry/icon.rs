use crate::ui::retained_host::host_contract::data::FrameRect;

use super::metrics::{
    ALERT_ICON_EDGE, ALERT_ICON_MARK_EDGE, ALERT_PADDING_X, alert_bounded_extent,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_icon_frame(
    rect: &FrameRect,
) -> FrameRect {
    let width = alert_bounded_extent(rect.width);
    let height = alert_bounded_extent(rect.height);
    let edge = ALERT_ICON_EDGE.min(width).min(height);
    let inset = ALERT_PADDING_X.min((width - edge).max(0.0));
    FrameRect {
        x: rect.x + inset,
        y: rect.y + (height - edge) * 0.5,
        width: edge,
        height: edge,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_icon_mark_frame(
    frame: &FrameRect,
) -> FrameRect {
    let edge = ALERT_ICON_MARK_EDGE
        .min(alert_bounded_extent(frame.width))
        .min(alert_bounded_extent(frame.height));
    FrameRect {
        x: frame.x + (alert_bounded_extent(frame.width) - edge) * 0.5,
        y: frame.y + (alert_bounded_extent(frame.height) - edge) * 0.5,
        width: edge,
        height: edge,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_icon_and_mark_stay_inside_tight_alert_bounds() {
        let alert = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 0.4,
            height: 0.6,
        };
        let icon = alert_icon_frame(&alert);
        let mark = alert_icon_mark_frame(&icon);

        for frame in [icon, mark] {
            assert!(frame.x >= alert.x);
            assert!(frame.y >= alert.y);
            assert!(frame.right() <= alert.right());
            assert!(frame.bottom() <= alert.bottom());
        }
    }
}
