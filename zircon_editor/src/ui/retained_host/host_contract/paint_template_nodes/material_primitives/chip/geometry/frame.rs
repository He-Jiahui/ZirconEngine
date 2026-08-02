use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::identity::chip_is_small;
use super::metrics::{CHIP_MEDIUM_HEIGHT, CHIP_SMALL_HEIGHT, chip_bounded_extent};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let available_height = chip_bounded_extent(rect.height);
    let target_height = chip_height(node).min(available_height);
    FrameRect {
        x: rect.x,
        y: rect.y + (available_height - target_height) * 0.5,
        width: chip_bounded_extent(rect.width),
        height: target_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_corner_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    let half_extent = chip_bounded_extent(rect.width).min(chip_bounded_extent(rect.height)) * 0.5;
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    if configured.is_finite() && configured > 0.0 {
        configured.min(half_extent)
    } else {
        half_extent
    }
}

fn chip_height(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        CHIP_SMALL_HEIGHT
    } else {
        CHIP_MEDIUM_HEIGHT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_frame_stays_inside_tight_parent_bounds() {
        let parent = FrameRect {
            x: 10.4,
            y: 20.8,
            width: 0.4,
            height: 0.6,
        };
        let frame = chip_frame(&TemplatePaneNodeData::default(), &parent);

        assert!(frame.x >= parent.x);
        assert!(frame.y >= parent.y);
        assert!(frame.right() <= parent.right());
        assert!(frame.bottom() <= parent.bottom());
    }

    #[test]
    fn chip_corner_radius_does_not_exceed_narrow_frame_bounds() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 20.0,
        };

        assert_eq!(
            chip_corner_radius(&TemplatePaneNodeData::default(), &rect),
            1.0
        );
    }
}
