use super::super::data::{FrameRect, HostWindowPresentationData};
use super::{debug_refresh_overlay_frame, presentation_top_bar_frame};

#[test]
fn debug_refresh_overlay_frame_uses_top_right_marker_geometry() {
    let top_bar = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 38.0,
    };

    let frame = debug_refresh_overlay_frame(&top_bar, "FPS 60").unwrap();

    assert_eq!(frame.y, 6.0);
    assert_eq!(frame.height, 26.0);
    assert!(frame.x > 0.0);
    assert!(frame.x + frame.width <= top_bar.width);
}

#[test]
fn debug_refresh_overlay_frame_uses_runtime_text_measurement() {
    let top_bar = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 600.0,
        height: 38.0,
    };

    let narrow = debug_refresh_overlay_frame(&top_bar, "iiiiiiiiiiii").unwrap();
    let wide = debug_refresh_overlay_frame(&top_bar, "WWWWWWWWWWWW").unwrap();

    assert!(
        wide.width > narrow.width + 8.0,
        "same-character-count diagnostics markers should follow runtime glyph width, narrow={narrow:?}, wide={wide:?}"
    );
}

#[test]
fn debug_refresh_overlay_frame_stays_inside_a_compact_top_bar() {
    let top_bar = FrameRect {
        x: 4.0,
        y: 8.0,
        width: 160.0,
        height: 18.0,
    };

    let frame = debug_refresh_overlay_frame(&top_bar, "FPS 60").unwrap();

    assert_eq!(frame.y, top_bar.y + 6.0);
    assert_eq!(frame.height, 12.0);
    assert!(frame.y + frame.height <= top_bar.y + top_bar.height);
}

#[test]
fn debug_refresh_overlay_frame_hides_when_top_bar_cannot_fit_its_inset() {
    let top_bar = FrameRect {
        x: 4.0,
        y: 8.0,
        width: 160.0,
        height: 6.0,
    };

    assert_eq!(debug_refresh_overlay_frame(&top_bar, "FPS 60"), None);
}

#[test]
fn presentation_top_bar_frame_uses_scene_layout_height_before_fallback() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.layout.center_band_frame = FrameRect {
        x: 0.0,
        y: 58.0,
        width: 200.0,
        height: 100.0,
    };

    let frame = presentation_top_bar_frame(200, 120, &presentation);

    assert_eq!(frame.height, 58.0);
}

#[test]
fn presentation_top_bar_frame_falls_back_for_empty_layout() {
    let frame = presentation_top_bar_frame(200, 120, &HostWindowPresentationData::default());

    assert_eq!(frame.height, 30.0);
}
