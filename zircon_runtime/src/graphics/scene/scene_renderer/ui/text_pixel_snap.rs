use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) fn text_origin_device_px(value: f32) -> f32 {
    if value.is_finite() {
        value.round()
    } else {
        0.0
    }
}

pub(super) fn text_frame_device_origin(frame: UiFrame) -> UiFrame {
    UiFrame::new(
        text_origin_device_px(frame.x),
        text_origin_device_px(frame.y),
        frame.width,
        frame.height,
    )
}

pub(super) fn text_glyph_device_frame(frame: UiFrame) -> UiFrame {
    UiFrame::new(
        text_origin_device_px(frame.x),
        text_origin_device_px(frame.y),
        frame.width,
        frame.height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_origin_device_px_rounds_finite_values() {
        assert_eq!(text_origin_device_px(12.49), 12.0);
        assert_eq!(text_origin_device_px(12.5), 13.0);
    }

    #[test]
    fn text_origin_device_px_drops_non_finite_values() {
        assert_eq!(text_origin_device_px(f32::NAN), 0.0);
        assert_eq!(text_origin_device_px(f32::INFINITY), 0.0);
    }

    #[test]
    fn text_frame_device_origin_preserves_extent() {
        let frame = text_frame_device_origin(UiFrame::new(3.6, 7.4, 120.0, 24.0));

        assert_eq!(frame, UiFrame::new(4.0, 7.0, 120.0, 24.0));
    }

    #[test]
    fn text_glyph_device_frame_snaps_bitmap_origin_only() {
        let frame = text_glyph_device_frame(UiFrame::new(10.58, 18.49, 9.5, 13.25));

        assert_eq!(frame, UiFrame::new(11.0, 18.0, 9.5, 13.25));
    }
}
