use super::super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paint_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: rect.width.max(1.0),
        height: rect.height.max(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_panel_preserves_fractional_post_dpi_geometry() {
        let rect = paint_rect(&FrameRect {
            x: 28.25,
            y: 36.5,
            width: 300.75,
            height: 420.25,
        });

        assert_eq!(rect.x, 28.25);
        assert_eq!(rect.y, 36.5);
        assert_eq!(rect.width, 300.75);
        assert_eq!(rect.height, 420.25);
    }
}
