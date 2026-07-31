use super::super::super::data::FrameRect;

const STATUS_ICON_CANVAS_SIZE: f32 = 16.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_status_glyph_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn centered_rect(
    rect: &FrameRect,
    size: f32,
) -> FrameRect {
    let width = size.min(rect.width.max(0.0)).max(0.0);
    let height = size.min(rect.height.max(0.0)).max(0.0);
    FrameRect {
        x: rect.x + (rect.width - width).max(0.0) * 0.5,
        y: rect.y + (rect.height - height).max(0.0) * 0.5,
        width,
        height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn local_rect(
    origin: &FrameRect,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> FrameRect {
    let scale_x = origin.width.max(0.0) / STATUS_ICON_CANVAS_SIZE;
    let scale_y = origin.height.max(0.0) / STATUS_ICON_CANVAS_SIZE;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: width * scale_x,
        height: height * scale_y,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn centered_canvas_rect(
    rect: &FrameRect,
    size: f32,
) -> FrameRect {
    let scale = rect.width.min(rect.height).max(0.0) / STATUS_ICON_CANVAS_SIZE;
    centered_rect(rect, size * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_icon_local_rect_scales_from_canonical_canvas() {
        let origin = FrameRect {
            x: 2.0,
            y: 4.0,
            width: 20.0,
            height: 24.0,
        };

        let rect = local_rect(&origin, 4.0, 8.0, 2.0, 4.0);

        assert!((rect.x - 7.0).abs() < 0.001);
        assert!((rect.y - 16.0).abs() < 0.001);
        assert!((rect.width - 2.5).abs() < 0.001);
        assert!((rect.height - 6.0).abs() < 0.001);
    }

    #[test]
    fn status_icon_centered_canvas_rect_scales_canonical_size() {
        let origin = FrameRect {
            x: 1.0,
            y: 3.0,
            width: 20.0,
            height: 24.0,
        };

        let rect = centered_canvas_rect(&origin, 4.0);

        assert!((rect.x - 8.5).abs() < 0.001);
        assert!((rect.y - 12.5).abs() < 0.001);
        assert!((rect.width - 5.0).abs() < 0.001);
        assert!((rect.height - 5.0).abs() < 0.001);
    }
}
