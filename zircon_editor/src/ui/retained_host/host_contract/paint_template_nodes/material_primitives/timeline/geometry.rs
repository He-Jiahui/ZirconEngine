use super::super::super::super::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_geometry::{
    bounded_extent, inward_pixel_aligned_rect,
};

pub(super) fn centered_square(rect: &FrameRect) -> FrameRect {
    let width = bounded_extent(rect.width);
    let height = bounded_extent(rect.height);
    let size = width.min(height);
    inward_pixel_aligned_rect(&FrameRect {
        x: rect.x + (width - size) * 0.5,
        y: rect.y + (height - size) * 0.5,
        width: size,
        height: size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_square_does_not_expand_a_collapsed_timeline_slot() {
        let square = centered_square(&FrameRect {
            x: 12.0,
            y: 8.0,
            width: 0.0,
            height: 24.0,
        });

        assert_eq!(square.width, 0.0);
        assert_eq!(square.height, 0.0);
    }
}
