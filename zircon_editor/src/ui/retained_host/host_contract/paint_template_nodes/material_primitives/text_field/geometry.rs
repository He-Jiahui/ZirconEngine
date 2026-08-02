use super::super::super::super::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_geometry::inward_pixel_aligned_rect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    inward_pixel_aligned_rect(rect)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_field_alignment_stays_inside_fractional_declared_bounds() {
        let declared = FrameRect {
            x: 12.3,
            y: 8.4,
            width: 95.2,
            height: 30.5,
        };

        let aligned = pixel_aligned_rect(&declared);

        assert_eq!(aligned.x, 13.0);
        assert_eq!(aligned.y, 9.0);
        assert_eq!(aligned.width, 94.0);
        assert_eq!(aligned.height, 29.0);
        assert!(aligned.right() <= declared.right());
        assert!(aligned.bottom() <= declared.bottom());
    }
}
