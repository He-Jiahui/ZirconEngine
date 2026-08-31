use super::super::super::data::FrameRect;
use super::metrics::axis_value_field_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_rect(
    rect: &FrameRect,
) -> FrameRect {
    let metrics = axis_value_field_metrics();
    let height = rect.height.min(metrics.max_height).max(0.0);
    FrameRect {
        x: rect.x,
        y: rect.y + (rect.height - height).max(0.0) * 0.5,
        width: rect.width.max(0.0),
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_field_preserves_fractional_post_dpi_geometry() {
        let rect = axis_field_rect(&FrameRect {
            x: 12.25,
            y: 7.5,
            width: 81.75,
            height: 31.5,
        });

        assert_eq!(rect.x, 12.25);
        assert_eq!(rect.width, 81.75);
        assert!(rect.y.fract() != 0.0);
        assert!(rect.height > 0.0);
    }
}
