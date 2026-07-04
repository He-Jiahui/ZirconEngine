use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime::core::framework::render::{
    ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphScript,
};
use zircon_runtime::ui::surface::{layout_text, shape_text_line};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextDirection, UiTextOverflow, UiTextRange, UiTextRunPaintStyle, UiTextWrap},
};

use super::{layout_text_run, layout_text_run_with_smoothing, runtime_positioned_glyphs};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_text::font::{font_for_face, HostTextFontFace};
use crate::ui::retained_host::host_contract::paint_theme::HostTextSmoothing;

#[test]
fn runtime_positioned_glyphs_use_runtime_grapheme_advances_when_widths_match() {
    let text = "Wi";
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, 3.0, 2.0);
    let host_width = super::host_glyph_run_width(&host_glyphs, HostTextFontFace::Ui)
        .expect("host glyph run width");
    let first_host_advance = super::glyph_cursor_x(&host_glyphs[1], HostTextFontFace::Ui)
        - super::glyph_cursor_x(&host_glyphs[0], HostTextFontFace::Ui);
    let second_adjusted_advance = (host_width - first_host_advance - 0.05).max(0.25);
    let adjusted_advances = vec![first_host_advance + 0.05, second_adjusted_advance];

    let glyphs = runtime_positioned_glyphs(
        text,
        &adjusted_advances,
        &[],
        HostTextFontFace::Ui,
        13.0,
        3.0,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), 2);
    let first_cursor = glyph_cursor_x(&glyphs[0]);
    let second_cursor = glyph_cursor_x(&glyphs[1]);
    assert!(
        (second_cursor - first_cursor - adjusted_advances[0]).abs() < 0.01,
        "host text glyph cursor advance must come from runtime layout"
    );
}

#[test]
fn runtime_positioned_glyphs_prefer_matching_shaped_glyph_positions() {
    let text = "editor";
    let start_x = 3.0;
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, start_x, 2.0);
    let host_advances = host_advances_for_text(text, &host_glyphs);
    let mut shaped_glyphs =
        shaped_glyphs_from_host_layout(text, &host_glyphs, HostTextFontFace::Ui, start_x);
    shaped_glyphs[1].x += 0.05;

    let glyphs = runtime_positioned_glyphs(
        text,
        &host_advances,
        &shaped_glyphs,
        HostTextFontFace::Ui,
        13.0,
        start_x,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), host_glyphs.len());
    assert!((glyphs[1].origin_x - (start_x + shaped_glyphs[1].x)).abs() < 0.01);
    assert!(
        (glyphs[1].origin_x - super::glyph_cursor_x(&host_glyphs[1], HostTextFontFace::Ui)).abs()
            > 0.03,
        "matching shaped glyph positions must not be discarded and recomputed from host advances"
    );
}

#[test]
fn runtime_positioned_glyphs_reject_matching_shaped_positions_with_local_jitter() {
    let text = "folder-open.svg";
    let start_x = 3.0;
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, start_x, 2.0);
    let host_advances = host_advances_for_text(text, &host_glyphs);
    let mut shaped_glyphs =
        shaped_glyphs_from_host_layout(text, &host_glyphs, HostTextFontFace::Ui, start_x);
    shaped_glyphs[1].x += 0.25;

    let glyphs = runtime_positioned_glyphs(
        text,
        &host_advances,
        &shaped_glyphs,
        HostTextFontFace::Ui,
        13.0,
        start_x,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), host_glyphs.len());
    for (projected, natural) in glyphs.iter().zip(host_glyphs.iter()) {
        assert!((projected.x - natural.x).abs() < 0.01);
    }
}

#[test]
fn runtime_positioned_glyphs_accepts_matching_shaped_positions_that_shift_raster_phase() {
    let text = "folder-open.svg";
    let start_x = 3.93;
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, start_x, 2.0);
    let host_advances = host_advances_for_text(text, &host_glyphs);
    let mut shaped_glyphs =
        shaped_glyphs_from_host_layout(text, &host_glyphs, HostTextFontFace::Ui, start_x);
    shaped_glyphs[0].x += 0.05;

    let glyphs = runtime_positioned_glyphs(
        text,
        &host_advances,
        &shaped_glyphs,
        HostTextFontFace::Ui,
        13.0,
        start_x,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), host_glyphs.len());
    let natural_origin = super::glyph_cursor_x(&host_glyphs[0], HostTextFontFace::Ui);
    let shaped_origin = start_x + shaped_glyphs[0].x;
    assert!(
        (glyphs[0].origin_x - shaped_origin).abs() < 0.01,
        "matching shaped positions should keep their pen origin even when the retained raster bin changes"
    );
    assert!(
        (glyphs[0].origin_x - natural_origin).abs() > 0.03,
        "this regression must cover the old fallback-to-natural-origin path"
    );
}

#[test]
fn runtime_positioned_glyphs_reject_mismatched_shaped_glyph_ids() {
    let text = "editor";
    let start_x = 3.0;
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, start_x, 2.0);
    let host_advances = host_advances_for_text(text, &host_glyphs);
    let mut shaped_glyphs =
        shaped_glyphs_from_host_layout(text, &host_glyphs, HostTextFontFace::Ui, start_x);
    shaped_glyphs[1].glyph_id = shaped_glyphs[1].glyph_id.saturating_add(1);
    shaped_glyphs[1].x += 0.25;

    let glyphs = runtime_positioned_glyphs(
        text,
        &host_advances,
        &shaped_glyphs,
        HostTextFontFace::Ui,
        13.0,
        start_x,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), host_glyphs.len());
    for (projected, natural) in glyphs.iter().zip(host_glyphs.iter()) {
        assert!((projected.x - natural.x).abs() < 0.01);
    }
}

#[test]
fn runtime_positioned_glyphs_fall_back_to_host_spacing_when_runtime_advances_mismatch() {
    let text = "editor";
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, 3.0, 2.0);
    let inflated_advances = vec![13.0; text.graphemes(true).count()];
    let glyphs = runtime_positioned_glyphs(
        text,
        &inflated_advances,
        &[],
        HostTextFontFace::Ui,
        13.0,
        3.0,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), host_glyphs.len());
    for (projected, natural) in glyphs.iter().zip(host_glyphs.iter()) {
        assert!((projected.x - natural.x).abs() < 0.01);
    }
}

#[test]
fn runtime_positioned_glyphs_rejects_per_grapheme_jitter_even_when_total_width_matches() {
    let text = "editor base.zui";
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, 3.0, 2.0);
    let graphemes = text.grapheme_indices(true).collect::<Vec<_>>();
    let host_advances =
        super::host_grapheme_advances(&host_glyphs, &graphemes, HostTextFontFace::Ui)
            .expect("host grapheme advances");
    let mut jittered_advances = host_advances.clone();
    jittered_advances[0] += 2.0;
    jittered_advances[1] = (jittered_advances[1] - 2.0).max(0.0);
    assert!(
        !super::runtime_advances_match_host_layout(
            &host_glyphs,
            &graphemes,
            &jittered_advances,
            HostTextFontFace::Ui,
        ),
        "a matching total width must not hide per-grapheme jitter"
    );

    let glyphs = runtime_positioned_glyphs(
        text,
        &jittered_advances,
        &[],
        HostTextFontFace::Ui,
        13.0,
        3.0,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), host_glyphs.len());
    for (projected, natural) in glyphs.iter().zip(host_glyphs.iter()) {
        assert!((projected.x - natural.x).abs() < 0.01);
    }
}

#[test]
fn runtime_positioned_glyphs_rejects_subpixel_tab_label_jitter() {
    let text = "editor base.zui";
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, 3.0, 2.0);
    let graphemes = text.grapheme_indices(true).collect::<Vec<_>>();
    let host_advances =
        super::host_grapheme_advances(&host_glyphs, &graphemes, HostTextFontFace::Ui)
            .expect("host grapheme advances");
    let mut jittered_advances = host_advances.clone();
    jittered_advances[0] += 0.75;
    jittered_advances[1] = (jittered_advances[1] - 0.75).max(0.0);

    assert!(
        !super::runtime_advances_match_host_layout(
            &host_glyphs,
            &graphemes,
            &jittered_advances,
            HostTextFontFace::Ui,
        ),
        "subpixel-sized local jitter is visible in editor tab labels"
    );

    let glyphs = runtime_positioned_glyphs(
        text,
        &jittered_advances,
        &[],
        HostTextFontFace::Ui,
        13.0,
        3.0,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), host_glyphs.len());
    for (projected, natural) in glyphs.iter().zip(host_glyphs.iter()) {
        assert!((projected.x - natural.x).abs() < 0.01);
    }
}

#[test]
fn runtime_positioned_glyphs_rejects_quarter_pixel_tab_label_jitter() {
    let text = "folder-open.svg";
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, 3.0, 2.0);
    let graphemes = text.grapheme_indices(true).collect::<Vec<_>>();
    let host_advances =
        super::host_grapheme_advances(&host_glyphs, &graphemes, HostTextFontFace::Ui)
            .expect("host grapheme advances");
    let mut jittered_advances = host_advances.clone();
    jittered_advances[0] += 0.25;
    jittered_advances[1] = (jittered_advances[1] - 0.25).max(0.0);

    assert!(
        !super::runtime_advances_match_host_layout(
            &host_glyphs,
            &graphemes,
            &jittered_advances,
            HostTextFontFace::Ui,
        ),
        "quarter-pixel local jitter is visible in editor tab labels"
    );

    let glyphs = runtime_positioned_glyphs(
        text,
        &jittered_advances,
        &[],
        HostTextFontFace::Ui,
        13.0,
        3.0,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), host_glyphs.len());
    for (projected, natural) in glyphs.iter().zip(host_glyphs.iter()) {
        assert!((projected.x - natural.x).abs() < 0.01);
    }
}

#[test]
fn runtime_positioned_glyphs_rejects_one_eighth_pixel_tab_label_jitter() {
    let text = "editor base.zui";
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, 3.0, 2.0);
    let graphemes = text.grapheme_indices(true).collect::<Vec<_>>();
    let host_advances =
        super::host_grapheme_advances(&host_glyphs, &graphemes, HostTextFontFace::Ui)
            .expect("host grapheme advances");
    let mut jittered_advances = host_advances.clone();
    jittered_advances[0] += 0.125;
    jittered_advances[1] = (jittered_advances[1] - 0.125).max(0.0);

    assert!(
        !super::runtime_advances_match_host_layout(
            &host_glyphs,
            &graphemes,
            &jittered_advances,
            HostTextFontFace::Ui,
        ),
        "one-eighth-pixel local jitter is visible in small editor tab labels"
    );

    let glyphs = runtime_positioned_glyphs(
        text,
        &jittered_advances,
        &[],
        HostTextFontFace::Ui,
        13.0,
        3.0,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), host_glyphs.len());
    for (projected, natural) in glyphs.iter().zip(host_glyphs.iter()) {
        assert!((projected.x - natural.x).abs() < 0.01);
    }
}

#[test]
fn runtime_positioned_glyphs_rejects_cumulative_subpixel_phase_drift() {
    let text = "editor base.zui";
    let start_x = 3.0;
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, start_x, 2.0);
    let graphemes = text.grapheme_indices(true).collect::<Vec<_>>();
    let host_advances =
        super::host_grapheme_advances(&host_glyphs, &graphemes, HostTextFontFace::Ui)
            .expect("host grapheme advances");
    let mut drifted_advances = host_advances.clone();
    for advance in drifted_advances.iter_mut().take(6) {
        *advance += 0.05;
    }
    for advance in drifted_advances.iter_mut().skip(6).take(6) {
        *advance = (*advance - 0.05).max(0.0);
    }

    assert!(
        super::runtime_advances_match_host_layout(
            &host_glyphs,
            &graphemes,
            &drifted_advances,
            HostTextFontFace::Ui,
        ),
        "per-grapheme tolerance alone accepts small local changes"
    );
    assert!(
        !super::runtime_advances_preserve_retained_raster_bins(
            &host_glyphs,
            &graphemes,
            &drifted_advances,
            HostTextFontFace::Ui,
            start_x,
            HostTextSmoothing::Subpixel,
        ),
        "cumulative drift must not move later editor-label glyphs into different raster bins"
    );

    let glyphs = runtime_positioned_glyphs(
        text,
        &drifted_advances,
        &[],
        HostTextFontFace::Ui,
        13.0,
        start_x,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), host_glyphs.len());
    for (projected, natural) in glyphs.iter().zip(host_glyphs.iter()) {
        assert!((projected.x - natural.x).abs() < 0.01);
    }
}

#[test]
fn runtime_positioned_glyphs_keep_pen_origin_for_subpixel_phase() {
    let text = "editor base.zui";
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, 3.25, 2.0);
    let host_advances = super::host_grapheme_advances(
        &host_glyphs,
        &text.grapheme_indices(true).collect::<Vec<_>>(),
        HostTextFontFace::Ui,
    )
    .expect("host grapheme advances");

    let glyphs = runtime_positioned_glyphs(
        text,
        &host_advances,
        &[],
        HostTextFontFace::Ui,
        13.0,
        3.25,
        2.0,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(glyphs.len(), host_glyphs.len());
    for (projected, natural) in glyphs.iter().zip(host_glyphs.iter()) {
        let natural_origin = super::glyph_cursor_x(natural, HostTextFontFace::Ui);
        assert!((projected.origin_x - natural_origin).abs() < 0.01);
        assert!((projected.x - natural.x).abs() < 0.01);
    }
}

#[test]
fn retained_text_run_carries_runtime_projected_spacing() {
    let rect = FrameRect {
        x: 5.0,
        y: 4.0,
        width: 200.0,
        height: 22.0,
    };
    let layout = layout_text_run(&rect, "Preview", 13.0, 16.0, UiTextRunPaintStyle::default());

    assert_eq!(layout.display_text, "Preview");
    assert_eq!(layout.font_face, HostTextFontFace::Ui);
    assert!(!layout.glyphs.is_empty());
}

#[test]
fn runtime_phase_guard_uses_alpha_subpixel_bins_for_grayscale_glyphs() {
    let text = "Wi";
    let probe_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, 0.0, 2.0);
    let probe_second_origin = super::glyph_cursor_x(&probe_glyphs[1], HostTextFontFace::Ui);
    let start_x = 20.49 - probe_second_origin.rem_euclid(1.0);
    let host_glyphs = super::fontdue_glyph_layout(text, HostTextFontFace::Ui, 13.0, start_x, 2.0);
    let graphemes = text.grapheme_indices(true).collect::<Vec<_>>();
    let mut advances =
        super::host_grapheme_advances(&host_glyphs, &graphemes, HostTextFontFace::Ui)
            .expect("host grapheme advances");
    let first_origin = super::glyph_cursor_x(&host_glyphs[0], HostTextFontFace::Ui);
    let second_origin = super::glyph_cursor_x(&host_glyphs[1], HostTextFontFace::Ui);
    advances[0] = second_origin.floor() + 0.51 - first_origin;

    assert!(
        super::runtime_advances_preserve_retained_raster_bins(
            &host_glyphs,
            &graphemes,
            &advances,
            HostTextFontFace::Ui,
            start_x,
            HostTextSmoothing::Grayscale,
        ),
        "grayscale alpha placement should keep the same 8-bin phase guard as subpixel positioning"
    );
    assert!(
        super::runtime_advances_preserve_retained_raster_bins(
            &host_glyphs,
            &graphemes,
            &advances,
            HostTextFontFace::Ui,
            start_x,
            HostTextSmoothing::Subpixel,
        ),
        "20.49px and 20.51px share the retained 8-bin phase"
    );
}

#[test]
fn retained_text_run_snaps_fractional_line_origin_before_glyph_spacing() {
    let fractional_rect = FrameRect {
        x: 8.875,
        y: 4.0,
        width: 200.0,
        height: 22.0,
    };
    let snapped_rect = FrameRect {
        x: 9.0,
        ..fractional_rect
    };

    let fractional = layout_text_run(
        &fractional_rect,
        "editor base.zui",
        13.0,
        16.0,
        UiTextRunPaintStyle::default(),
    );
    let snapped = layout_text_run(
        &snapped_rect,
        "editor base.zui",
        13.0,
        16.0,
        UiTextRunPaintStyle::default(),
    );

    assert_eq!(fractional.display_text, snapped.display_text);
    assert_eq!(fractional.glyphs.len(), snapped.glyphs.len());
    for (left, right) in fractional.glyphs.iter().zip(snapped.glyphs.iter()) {
        assert!(
            (left.origin_x - right.origin_x).abs() < 0.01,
            "fractional editor label line origin should snap once before preserving glyph spacing: fractional={left:?}, snapped={right:?}"
        );
    }
}

#[test]
fn retained_text_run_preserves_fractional_line_origin_for_subpixel_smoothing() {
    let fractional_rect = FrameRect {
        x: 8.875,
        y: 4.0,
        width: 200.0,
        height: 22.0,
    };
    let integer_rect = FrameRect {
        x: 9.0,
        ..fractional_rect
    };

    let fractional = layout_text_run_with_smoothing(
        &fractional_rect,
        "editor base.zui",
        13.0,
        16.0,
        HostTextFontFace::Ui,
        HostTextSmoothing::Subpixel,
    );
    let integer = layout_text_run_with_smoothing(
        &integer_rect,
        "editor base.zui",
        13.0,
        16.0,
        HostTextFontFace::Ui,
        HostTextSmoothing::Subpixel,
    );

    assert_eq!(fractional.display_text, integer.display_text);
    assert_eq!(fractional.glyphs.len(), integer.glyphs.len());
    for (left, right) in fractional.glyphs.iter().zip(integer.glyphs.iter()) {
        assert!(
            (right.origin_x - left.origin_x - 0.125).abs() < 0.01,
            "explicit subpixel text should preserve fractional line origin instead of collapsing to the grayscale snapped origin: fractional={left:?}, integer={right:?}"
        );
    }
}

#[test]
fn retained_text_run_uses_runtime_ellipsis_for_narrow_editor_labels() {
    let source = "folder-open.svg";
    let style = super::runtime_text_style_for_face(
        HostTextFontFace::Ui,
        13.0,
        16.0,
        UiTextWrap::None,
        UiTextOverflow::Ellipsis,
    );
    let ellipsis_width = shape_text_line("\u{2026}", &style)
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
fn runtime_single_line_text_uses_runtime_shaped_glyph_advances() {
    let rect = FrameRect {
        x: 5.0,
        y: 4.0,
        width: 200.0,
        height: 22.0,
    };
    let line = super::runtime_single_line_text(&rect, "Preview", 13.0, 16.0, HostTextFontFace::Ui);
    let style = super::runtime_text_style_for_face(
        HostTextFontFace::Ui,
        13.0,
        16.0,
        UiTextWrap::None,
        UiTextOverflow::Ellipsis,
    );
    let shaped = shape_text_line(line.text.as_str(), &style);
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

fn glyph_cursor_x(glyph: &super::RuntimeTextGlyph) -> f32 {
    glyph.origin_x
}

fn host_advances_for_text(text: &str, host_glyphs: &[fontdue::layout::GlyphPosition]) -> Vec<f32> {
    super::host_grapheme_advances(
        host_glyphs,
        &text.grapheme_indices(true).collect::<Vec<_>>(),
        HostTextFontFace::Ui,
    )
    .expect("host grapheme advances")
}

fn shaped_glyphs_from_host_layout(
    text: &str,
    host_glyphs: &[fontdue::layout::GlyphPosition],
    font_face: HostTextFontFace,
    start_x: f32,
) -> Vec<ShapedGlyph> {
    host_glyphs
        .iter()
        .map(|glyph| {
            let visual_range = grapheme_range_for_byte_offset(text, glyph.byte_offset);
            let metrics = font_for_face(font_face)
                .expect("retained-host font")
                .metrics_indexed(glyph.key.glyph_index, glyph.key.px);
            ShapedGlyph {
                glyph_id: glyph.key.glyph_index as u32,
                font_id: None,
                source_range: visual_range,
                visual_range,
                advance: metrics.advance_width.max(0.0),
                x: super::glyph_cursor_x(glyph, font_face) - start_x,
                y: glyph.y,
                offset_x: 0.0,
                offset_y: 0.0,
                direction: UiTextDirection::LeftToRight,
                cluster_flags: ShapedGlyphClusterFlags {
                    cluster_start: true,
                    ..ShapedGlyphClusterFlags::default()
                },
                rotation: ShapedGlyphRotation::None,
                script: ShapedGlyphScript::default(),
            }
        })
        .collect()
}

fn grapheme_range_for_byte_offset(text: &str, byte_offset: usize) -> UiTextRange {
    text.grapheme_indices(true)
        .find_map(|(start, grapheme)| {
            let end = start + grapheme.len();
            (byte_offset >= start && byte_offset < end).then_some(UiTextRange { start, end })
        })
        .unwrap_or(UiTextRange {
            start: text.len(),
            end: text.len(),
        })
}
