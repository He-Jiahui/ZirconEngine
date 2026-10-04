use super::runtime_lines::RuntimeTextLine;
use super::{display_text_from_lines, layout_text_run, layout_text_run_with_smoothing};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_text::font::HostTextFontFace;
use crate::ui::retained_host::host_contract::paint_theme::HostTextSmoothing;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

fn runtime_text_line(text: &str) -> RuntimeTextLine {
    RuntimeTextLine {
        text: text.to_string(),
        frame_x: 0.0,
        frame_y: 0.0,
        frame_width: 0.0,
        frame_height: 0.0,
        artifact_line: None,
    }
}

#[test]
fn display_text_joins_runtime_visual_lines_without_relayout() {
    assert_eq!(
        display_text_from_lines(&[runtime_text_line("First"), runtime_text_line("Second")]),
        "First\nSecond"
    );
}

#[test]
fn layout_uses_runtime_artifact_faces_for_basic_text() {
    let layout = layout_text_run(
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 24.0,
        },
        "Runtime text",
        14.0,
        18.0,
        UiTextRunPaintStyle::default(),
    );

    assert_eq!(layout.display_text, "Runtime text");
    assert!(!layout.glyphs.is_empty());
    assert!(!layout.artifact_raster_faces.is_empty());
    assert!(layout.glyphs.iter().all(|glyph| {
        glyph.raster_face_index < layout.artifact_raster_faces.len()
            && glyph.physical_ppem == 14
            && glyph.origin_x.is_finite()
            && glyph.baseline_y.is_finite()
    }));
}

#[test]
fn grayscale_origin_snap_is_applied_before_artifact_projection() {
    let rect = FrameRect {
        x: 0.75,
        y: 0.0,
        width: 200.0,
        height: 24.0,
    };
    let layout = layout_text_run_with_smoothing(
        &rect,
        "Snap",
        14.0,
        18.0,
        HostTextFontFace::Ui,
        HostTextSmoothing::Grayscale,
    );
    let first = layout.glyphs.first().expect("runtime artifact glyph");
    assert_eq!(first.origin_x.fract(), 0.0);
}
