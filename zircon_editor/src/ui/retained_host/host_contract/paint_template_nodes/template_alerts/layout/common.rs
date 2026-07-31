use super::super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_alert_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn frame_is_within(
    inner: &FrameRect,
    outer: &FrameRect,
) -> bool {
    if !has_paintable_alert_extent(inner) || !has_paintable_alert_extent(outer) {
        return false;
    }

    let inner_right = inner.x + inner.width;
    let inner_bottom = inner.y + inner.height;
    let outer_right = outer.x + outer.width;
    let outer_bottom = outer.y + outer.height;
    inner_right.is_finite()
        && inner_bottom.is_finite()
        && outer_right.is_finite()
        && outer_bottom.is_finite()
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner_right <= outer_right
        && inner_bottom <= outer_bottom
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    if !has_paintable_alert_extent(rect) {
        return FrameRect {
            x: rect.x,
            y: rect.y,
            width: 0.0,
            height: 0.0,
        };
    }
    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;
    if !right.is_finite() || !bottom.is_finite() {
        return FrameRect {
            x: rect.x,
            y: rect.y,
            width: 0.0,
            height: 0.0,
        };
    }
    let x = rect.x.ceil();
    let y = rect.y.ceil();
    FrameRect {
        x,
        y,
        width: (right.floor() - x).max(0.0),
        height: (bottom.floor() - y).max(0.0),
    }
}

pub(super) fn centered_rect(rect: &FrameRect, left: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x: rect.x + left,
        y: rect.y + (rect.height - height).max(0.0) * 0.5,
        width,
        height,
    }
}

pub(super) fn fitted_centered_square(rect: &FrameRect, left: f32, desired_size: f32) -> FrameRect {
    let size = desired_size
        .min(rect.height.max(0.0))
        .min((rect.width - left).max(0.0));
    centered_rect(rect, left, size, size)
}

#[cfg(test)]
mod tests {
    use super::pixel_aligned_rect;
    use crate::ui::retained_host::host_contract::data::FrameRect;

    #[test]
    fn pixel_alignment_preserves_degenerate_extents_for_the_paint_guard() {
        let aligned = pixel_aligned_rect(&FrameRect {
            x: 4.4,
            y: 6.6,
            width: 0.4,
            height: -2.0,
        });

        assert_eq!(aligned.x, 5.0);
        assert_eq!(aligned.y, 7.0);
        assert_eq!(aligned.width, 0.0);
        assert_eq!(aligned.height, 0.0);
    }
}
