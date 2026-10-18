use super::*;

#[test]
fn retained_text_label_preserves_u0020_advance_in_painted_pixels() {
    // Exercise the retained painter at the 100% and 150% label sizes. Spaces
    // have no ink, but must still move the following glyph on the framebuffer.
    for scale in [1.0, 1.5] {
        let font_size = 13.0 * scale;
        let line_height = 16.0 * scale;
        let space_width = measure_runtime_text_width("\u{0020}", font_size);
        assert!(space_width > 0.0, "the label font must give U+0020 an advance");

        let (compact_left, compact_right) = label_ink_bounds("HH", font_size, line_height);
        for (text, spaces) in [("H H", 1), ("H  H", 2)] {
            let (left, right) = label_ink_bounds(text, font_size, line_height);
            assert_eq!(left, compact_left, "a space must not shift the first glyph");
            assert!(right > compact_right, "a space must move the following glyph");
            let painted_advance = right as f32 - compact_right as f32;
            let expected_advance = space_width * spaces as f32;
            assert!(
                (painted_advance - expected_advance).abs() <= 1.5,
                "{text:?} at scale {scale}: painted advance {painted_advance} must preserve the U+0020 advance {expected_advance}"
            );
        }
    }
}

fn label_ink_bounds(text: &str, font_size: f32, line_height: f32) -> (u32, u32) {
    const BACKGROUND: [u8; 4] = [17, 22, 26, 255];
    let mut frame = HostRgbaFrame::filled(256, 64, BACKGROUND);
    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x: 8.0,
            y: 8.0,
            width: 240.0,
            height: line_height + 8.0,
        },
        text,
        None,
        [224, 232, 238, 255],
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
    );

    let mut left = frame.width();
    let mut right = 0;
    for (index, pixel) in frame.as_bytes().chunks_exact(4).enumerate() {
        if pixel != BACKGROUND {
            let x = index as u32 % frame.width();
            left = left.min(x);
            right = right.max(x);
        }
    }
    assert!(left <= right, "the label must paint visible glyphs: {text:?}");
    (left, right)
}
