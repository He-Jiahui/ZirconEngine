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
        text_glyph_position_px(frame.x),
        text_glyph_position_px(frame.y),
        frame.width,
        frame.height,
    )
}

fn text_glyph_position_px(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "tests/text_pixel_snap.rs"]
mod tests;
