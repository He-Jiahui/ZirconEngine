use zircon_runtime::core::framework::render::CapturedFrame;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiRenderCommand, UiTextRenderMode},
};

pub(in super::super) fn assert_msdf_sharp_corner_pixels(
    samples: &[UiRenderCommand],
    capture: &CapturedFrame,
    background: &CapturedFrame,
) {
    let sdf = super::sample_by_node(samples, 107);
    let msdf = super::sample_by_node(samples, 123);
    assert_eq!(sdf.style.text_render_mode, UiTextRenderMode::Sdf);
    assert_eq!(msdf.style.text_render_mode, UiTextRenderMode::Msdf);
    assert_msdf_fixture_contract(sdf, msdf);
    let sdf_bounds = super::super::changed_pixel_bounds_in_frame(
        &capture.rgba,
        &background.rgba,
        capture.width,
        capture.height,
        sdf.frame,
        10,
    )
    .expect("SDF sharp-corner comparison must contain real framebuffer pixels");
    let msdf_bounds = super::super::changed_pixel_bounds_in_frame(
        &capture.rgba,
        &background.rgba,
        capture.width,
        capture.height,
        msdf.frame,
        10,
    )
    .expect("MSDF sharp-corner comparison must contain real framebuffer pixels");
    assert!(sdf_bounds.4 > 64 && msdf_bounds.4 > 64);
    let decode_delta = super::super::count_relative_pixel_differences(
        &capture.rgba,
        capture.width,
        capture.height,
        sharp_glyph_region(sdf.frame),
        sharp_glyph_region(msdf.frame),
        6,
    );
    assert!(
        decode_delta > 32,
        "SDF and MSDF must reach distinct real framebuffer decode paths in the common A/M/W sharp-glyph region; delta={decode_delta}"
    );
    let sdf_apex = sharp_a_apex_profile(sdf.frame, capture, background);
    let msdf_apex = sharp_a_apex_profile(msdf.frame, capture, background);
    assert!(
        msdf_apex.top_offset <= sdf_apex.top_offset,
        "MSDF must preserve the side-by-side A apex at least as high as SDF; sdf={sdf_apex:?}, msdf={msdf_apex:?}"
    );
}

fn assert_msdf_fixture_contract(sdf: &UiRenderCommand, msdf: &UiRenderCommand) {
    assert_eq!(sdf.text, msdf.text);
    assert_eq!(sdf.style.language, msdf.style.language);
    assert_fixture_close(sdf.frame.y, msdf.frame.y, "SDF/MSDF fixture y");
    assert_fixture_close(sdf.frame.width, msdf.frame.width, "SDF/MSDF fixture width");
    assert_fixture_close(
        sdf.frame.height,
        msdf.frame.height,
        "SDF/MSDF fixture height",
    );
}

fn assert_fixture_close(lhs: f32, rhs: f32, label: &str) {
    assert!(
        (lhs - rhs).abs() <= super::FRAME_EPSILON,
        "{label} must match: lhs={lhs}, rhs={rhs}"
    );
}

fn sharp_glyph_region(frame: UiFrame) -> UiFrame {
    const SHARP_GLYPH_REGION_WIDTH: f32 = 96.0;
    UiFrame::new(
        frame.x,
        frame.y,
        frame.width.min(SHARP_GLYPH_REGION_WIDTH),
        frame.height,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SharpApexProfile {
    top_offset: usize,
}

fn sharp_a_apex_profile(
    frame: UiFrame,
    capture: &CapturedFrame,
    background: &CapturedFrame,
) -> SharpApexProfile {
    const APEX_WIDTH: usize = 22;
    const APEX_ROWS: usize = 4;
    const CONTRAST_THRESHOLD: u8 = 30;
    let width = capture.width as usize;
    let height = capture.height as usize;
    let left = frame.x.max(0.0).floor() as usize;
    let top = frame.y.max(0.0).floor() as usize;
    let bottom = frame.bottom().max(0.0).ceil() as usize;
    let mut changed = Vec::new();
    for y in top.min(height)..bottom.min(height) {
        for x in left.min(width)..left.saturating_add(APEX_WIDTH).min(width) {
            let index = (y * width + x) * 4;
            let delta = capture.rgba[index..index + 4]
                .iter()
                .zip(&background.rgba[index..index + 4])
                .map(|(sample, baseline)| sample.abs_diff(*baseline))
                .max()
                .unwrap_or(0);
            if delta >= CONTRAST_THRESHOLD {
                changed.push((x, y));
            }
        }
    }
    let apex_top = changed
        .iter()
        .map(|(_, y)| *y)
        .min()
        .expect("sharp-corner A sample must have high-contrast framebuffer pixels");
    let high_contrast_pixels = changed
        .iter()
        .filter(|(_, y)| *y < apex_top.saturating_add(APEX_ROWS))
        .count();
    assert!(
        high_contrast_pixels >= APEX_ROWS,
        "sharp-corner A apex must remain visibly high contrast"
    );
    SharpApexProfile {
        top_offset: apex_top.saturating_sub(top),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sharp_glyph_region_caps_width_without_moving_the_glyph_origin() {
        assert_eq!(
            sharp_glyph_region(UiFrame::new(42.0, 594.0, 410.0, 58.0)),
            UiFrame::new(42.0, 594.0, 96.0, 58.0)
        );
    }
}
