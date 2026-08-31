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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_centered_slider_geometry_has_no_drawable_extent() {
        let invalid = centered_rect(8.0, 6.0, f32::NAN);

        assert_eq!((invalid.width, invalid.height), (0.0, 0.0));
        assert_eq!((invalid.x, invalid.y), (8.0, 6.0));
    }
}
