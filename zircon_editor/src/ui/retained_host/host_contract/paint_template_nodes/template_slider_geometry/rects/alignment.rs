use super::super::super::super::{data::FrameRect, paint_geometry::bounded_extent};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn centered_rect(
    center_x: f32,
    center_y: f32,
    size: f32,
) -> FrameRect {
    let size = bounded_extent(size);
    FrameRect {
        x: center_x - size * 0.5,
        y: center_y - size * 0.5,
        width: size,
        height: size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: bounded_extent(rect.width.round()),
        height: bounded_extent(rect.height.round()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_or_invalid_slider_alignment_has_no_drawable_extent() {
        let collapsed = pixel_aligned_rect(&FrameRect {
            x: 8.4,
            y: 6.6,
            width: 0.0,
            height: 0.0,
        });
        let invalid = centered_rect(8.0, 6.0, f32::NAN);

        assert_eq!((collapsed.width, collapsed.height), (0.0, 0.0));
        assert_eq!((invalid.width, invalid.height), (0.0, 0.0));
        assert_eq!((invalid.x, invalid.y), (8.0, 6.0));
    }
}
