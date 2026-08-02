use crate::ui::retained_host::host_contract::data::FrameRect;

use super::metrics::alert_bounded_extent;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: alert_bounded_extent(rect.width),
        height: alert_bounded_extent(rect.height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_root_stays_inside_tight_parent_bounds() {
        let parent = FrameRect {
            x: 10.4,
            y: 20.8,
            width: 0.4,
            height: 0.6,
        };
        let frame = alert_rect(&parent);

        assert!(frame.x >= parent.x);
        assert!(frame.y >= parent.y);
        assert!(frame.right() <= parent.right());
        assert!(frame.bottom() <= parent.bottom());
    }
}
