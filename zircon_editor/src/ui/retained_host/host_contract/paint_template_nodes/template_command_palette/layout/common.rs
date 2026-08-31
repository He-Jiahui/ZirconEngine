use super::super::super::super::data::FrameRect;
use super::metrics::command_palette_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn min_frame_extent() -> f32 {
    command_palette_metrics().min_frame_extent
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paint_rect(
    rect: &FrameRect,
) -> FrameRect {
    let min_frame_extent = min_frame_extent();
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: rect.width.max(min_frame_extent),
        height: rect.height.max(min_frame_extent),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_palette_preserves_fractional_post_dpi_geometry() {
        let rect = paint_rect(&FrameRect {
            x: 11.25,
            y: 17.5,
            width: 319.75,
            height: 241.25,
        });

        assert_eq!(rect.x, 11.25);
        assert_eq!(rect.y, 17.5);
        assert_eq!(rect.width, 319.75);
        assert_eq!(rect.height, 241.25);
    }
}

pub(super) fn symmetric_extent(inset: f32) -> f32 {
    inset * 2.0
}

pub(super) fn centered_offset(outer: f32, inner: f32) -> f32 {
    (outer - inner).max(0.0) * 0.5
}
