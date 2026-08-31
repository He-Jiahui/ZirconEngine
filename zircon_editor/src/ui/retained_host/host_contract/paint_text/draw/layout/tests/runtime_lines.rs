use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime::ui::surface::{layout_text, shape_text_line};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextRunPaintStyle, UiTextWrap},
};

use super::super::super::super::super::data::FrameRect;
use super::super::super::super::font::{runtime_text_style_for_face, HostTextFontFace};
use super::super::{layout_text_run, layout_text_run_with_layout_policy};
use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;

#[test]
fn runtime_single_line_text_uses_runtime_shaped_glyph_advances() {
    let rect = FrameRect {
        x: 5.0,
        y: 4.0,
        width: 200.0,
        height: 22.0,
    };
    let line =
        super::super::runtime_single_line_text(&rect, "Preview", 13.0, 16.0, HostTextFontFace::Ui);
    let style = runtime_text_style_for_face(
        HostTextFontFace::Ui,
        13.0,
        16.0,
        UiTextWrap::None,
        UiTextOverflow::Ellipsis,
    );
    let shaped = shape_text_line(line.text.as_str(), &style).expect("shape text");
    let shaped_width = shaped
        .lines
        .first()
        .map(|shaped_line| shaped_line.measured_width)
        .unwrap_or_default();
    let advance_width = line.glyph_advances.iter().sum::<f32>();

    assert_eq!(line.glyph_advances.len(), line.text.graphemes(true).count());
    assert!(advance_width > 0.0);
    assert!((advance_width - shaped_width).abs() < 1.0);
}

#[test]
fn retained_text_run_uses_runtime_ellipsis_for_narrow_editor_labels() {
    let source = "folder-open.svg";
    let style = runtime_text_style_for_face(
        HostTextFontFace::Ui,
        13.0,
        16.0,
        UiTextWrap::None,
        UiTextOverflow::Ellipsis,
    );
    let ellipsis_width = shape_text_line("\u{2026}", &style)
        .expect("shape ellipsis")
        .lines
        .first()
        .expect("ellipsis shaped line")
        .measured_width;
    let rect = FrameRect {
        x: 5.0,
        y: 4.0,
        width: ellipsis_width + 1.0,
        height: 22.0,
    };
    let runtime_layout = layout_text(
        source,
        &style,
        UiFrame::new(0.0, 0.0, rect.width, 16.0),
        None,
    );
    let runtime_line = runtime_layout.lines.first().expect("runtime text line");

    let layout = layout_text_run(&rect, source, 13.0, 16.0, UiTextRunPaintStyle::default());

    assert!(runtime_line.ellipsized);
    assert_ne!(layout.display_text, source);
    assert_eq!(layout.display_text, runtime_line.text);
    assert!(layout.display_text.contains('\u{2026}'));
    assert!(!layout.glyphs.is_empty());
    assert!(layout
        .glyphs
        .iter()
        .all(|glyph| glyph.x.is_finite() && glyph.origin_x.is_finite() && glyph.y.is_finite()));
}

#[test]
fn retained_text_run_preserves_runtime_word_wrapped_lines_for_body_copy() {
    let rect = FrameRect {
        x: 5.0,
        y: 4.0,
        width: 72.0,
        height: 48.0,
    };
    let layout = layout_text_run_with_layout_policy(
        &rect,
        "Alpha Bravo Charlie Delta",
        13.0,
        16.0,
        UiTextRunPaintStyle::default(),
        HostTextLayoutPolicy::WordWrap,
    );
    let line_origins = layout
        .glyphs
        .iter()
        .map(|glyph| glyph.y.round() as i32)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(layout.display_text.contains('\n'));
    assert!(line_origins.len() >= 2);
}
