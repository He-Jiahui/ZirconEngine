use std::sync::Arc;

use zircon_runtime::ui::surface::{layout_text, measure_text_size};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiTextOverflow, UiTextRunPaintStyle, UiTextWrap},
};

use super::draw::{draw_text_with_size_and_style, DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT};
use super::font::{
    font_face_for_paint_style, font_request_for_face, font_request_for_face_with_preferences,
    runtime_font_family_for_face, runtime_text_style_for_face, HostTextFontFace,
};
use super::raster::rasterize_cached_glyph;
use super::{measure_runtime_text_width, measure_runtime_text_width_with_style};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::{HostRecordedPaintKind, HostRgbaFrame};
use crate::ui::retained_host::host_contract::paint_theme::{
    HostTextPreferences, HostTextSmoothing, HostUtilityTabTextRole,
};

#[test]
fn glyph_raster_cache_reuses_bitmap_for_same_glyph_and_size() {
    let first = rasterize_cached_glyph(HostTextFontFace::Ui, 1, DEFAULT_FONT_SIZE, 3.0, 0.0);
    let second = rasterize_cached_glyph(HostTextFontFace::Ui, 1, DEFAULT_FONT_SIZE, 3.0, 0.0);

    assert_eq!(first.metrics.width, second.metrics.width);
    assert!(Arc::ptr_eq(&first.bitmap, &second.bitmap));
}

#[test]
fn retained_text_font_face_tracks_ui_and_code_styles() {
    assert_eq!(
        font_face_for_paint_style(UiTextRunPaintStyle::default()),
        HostTextFontFace::Ui
    );
    assert_eq!(
        font_face_for_paint_style(UiTextRunPaintStyle {
            strong: true,
            ..UiTextRunPaintStyle::default()
        }),
        HostTextFontFace::UiStrong
    );
    assert_eq!(
        font_face_for_paint_style(UiTextRunPaintStyle {
            code: true,
            strong: true,
            ..UiTextRunPaintStyle::default()
        }),
        HostTextFontFace::Mono
    );
}

#[test]
fn retained_ui_runtime_family_resolves_from_preferences_without_platform_paths() {
    let ui_family = runtime_font_family_for_face(HostTextFontFace::Ui);
    let mono_family = runtime_font_family_for_face(HostTextFontFace::Mono);

    assert!(!ui_family.trim().is_empty());
    assert!(!mono_family.trim().is_empty());
    assert!(!ui_family.contains('\\'));
    assert!(!mono_family.contains('\\'));
    assert_ne!(
        ui_family, mono_family,
        "ordinary UI text and code text must not collapse to one runtime family"
    );
}

#[test]
fn retained_text_font_request_uses_global_preferences() {
    let preferences = HostTextPreferences {
        ui_family: "ui-family".to_string(),
        ui_strong_family: "ui-strong-family".to_string(),
        code_family: "code-family".to_string(),
        utility_tab_text_role: HostUtilityTabTextRole::Ui,
        ui_weight: 410,
        strong_weight: 620,
        code_weight: 430,
        smoothing: HostTextSmoothing::Grayscale,
    };

    let ui = font_request_for_face_with_preferences(HostTextFontFace::Ui, &preferences);
    let strong = font_request_for_face_with_preferences(HostTextFontFace::UiStrong, &preferences);
    let mono = font_request_for_face_with_preferences(HostTextFontFace::Mono, &preferences);

    assert_eq!(ui.family, "ui-family");
    assert_eq!(ui.weight, 410);
    assert_eq!(strong.family, "ui-strong-family");
    assert_eq!(strong.weight, 620);
    assert_eq!(mono.family, "code-family");
    assert_eq!(mono.weight, 430);
}

#[test]
fn glyph_raster_cache_keeps_ui_and_mono_faces_separate() {
    let ui = rasterize_cached_glyph(HostTextFontFace::Ui, 1, DEFAULT_FONT_SIZE, 3.0, 0.0);
    let mono = rasterize_cached_glyph(HostTextFontFace::Mono, 1, DEFAULT_FONT_SIZE, 3.0, 0.0);

    assert!(!Arc::ptr_eq(&ui.bitmap, &mono.bitmap));
}

#[test]
fn text_draw_skips_disjoint_active_and_explicit_clips() {
    let mut frame = HostRgbaFrame::filled(64, 32, [0, 0, 0, 255]);
    frame.replace_paint_clip(Some(FrameRect {
        x: 0.0,
        y: 0.0,
        width: 8.0,
        height: 8.0,
    }));

    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x: 16.0,
            y: 16.0,
            width: 40.0,
            height: 12.0,
        },
        "Ready",
        Some(&FrameRect {
            x: 16.0,
            y: 16.0,
            width: 40.0,
            height: 12.0,
        }),
        [255, 255, 255, 255],
        DEFAULT_FONT_SIZE,
        DEFAULT_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
    );

    assert!(frame
        .as_bytes()
        .chunks_exact(4)
        .all(|pixel| pixel == [0, 0, 0, 255]));
}

#[test]
fn retained_text_records_runtime_single_line_ellipsis_projection() {
    let text = "a\u{0301}bc";
    let style = runtime_single_line_style(DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT);
    let (width, expected) = first_runtime_width_that_keeps_combining_grapheme(text, &style);
    let mut frame = HostRgbaFrame::recording_only(64, 24);

    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width,
            height: DEFAULT_LINE_HEIGHT,
        },
        text,
        None,
        [255, 255, 255, 255],
        DEFAULT_FONT_SIZE,
        DEFAULT_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
    );

    let commands = frame.into_recorded_commands();
    let recorded_text = commands
        .iter()
        .find_map(|command| match &command.kind {
            HostRecordedPaintKind::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .expect("recording should contain a text command");

    assert_eq!(recorded_text, expected);
    assert_ne!(recorded_text, text);
}

#[test]
fn retained_text_preserves_small_underscore_stroke_contrast() {
    let mut frame = HostRgbaFrame::filled(48, 20, [0, 0, 0, 255]);

    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x: 4.0,
            y: 2.0,
            width: 40.0,
            height: 12.0,
        },
        "_",
        None,
        [255, 255, 255, 255],
        9.0,
        12.0,
        UiTextRunPaintStyle::default(),
    );

    let max_luminance = frame
        .as_bytes()
        .chunks_exact(4)
        .map(|pixel| pixel[0].max(pixel[1]).max(pixel[2]))
        .max()
        .unwrap_or(0);

    assert!(
        max_luminance >= 128,
        "small underscores should remain visible after retained-host text raster downsampling, got max luminance {max_luminance}"
    );
}

#[test]
fn retained_text_draws_editor_crop_labels_with_runtime_ellipsis_pixels() {
    const BACKGROUND: [u8; 4] = [17, 22, 26, 255];
    let mut frame = HostRgbaFrame::filled(220, 72, BACKGROUND);

    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x: 8.0,
            y: 8.0,
            width: 136.0,
            height: 22.0,
        },
        "editor base.zui",
        None,
        [224, 232, 238, 255],
        13.0,
        16.0,
        UiTextRunPaintStyle::default(),
    );
    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x: 8.0,
            y: 40.0,
            width: 42.0,
            height: 22.0,
        },
        "folder-open.svg",
        None,
        [224, 232, 238, 255],
        13.0,
        16.0,
        UiTextRunPaintStyle::default(),
    );

    let painted_pixels = frame
        .as_bytes()
        .chunks_exact(4)
        .filter(|pixel| *pixel != BACKGROUND)
        .count();
    assert!(
        painted_pixels > 40,
        "retained-host text drawing should emit visible pixels for editor crop labels"
    );

    let mut recording = HostRgbaFrame::recording_only(96, 24);
    draw_text_with_size_and_style(
        &mut recording,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 42.0,
            height: 22.0,
        },
        "folder-open.svg",
        None,
        [224, 232, 238, 255],
        13.0,
        16.0,
        UiTextRunPaintStyle::default(),
    );
    let recorded_text = recording
        .into_recorded_commands()
        .into_iter()
        .find_map(|command| match command.kind {
            HostRecordedPaintKind::Text { text, .. } => Some(text),
            _ => None,
        })
        .expect("narrow retained-host label should record a text command");

    assert_ne!(recorded_text, "folder-open.svg");
    assert!(
        recorded_text.contains('\u{2026}'),
        "narrow retained-host labels should record runtime ellipsis text"
    );
}

#[test]
fn retained_text_editor_crop_labels_keep_stable_ink_spacing() {
    let full_label = editor_crop_ink_profile("editor base.zui", 8.875, 136.0);
    let shifted_full_label = editor_crop_ink_profile("editor base.zui", 8.925, 136.0);
    let narrow_label = editor_crop_ink_profile("folder-open.svg", 8.875, 42.0);

    assert!(full_label.painted_pixels > 100);
    assert!(narrow_label.painted_pixels > 30);
    assert!(
        full_label.max_internal_empty_columns <= 7,
        "full editor tab label should not contain a large unexpected blank run: {full_label:?}"
    );
    assert!(
        narrow_label.max_internal_empty_columns <= 5,
        "ellipsized editor tab label should keep compact readable spacing: {narrow_label:?}"
    );
    assert!(
        (shifted_full_label.ink_center_x - full_label.ink_center_x).abs() <= 1.0,
        "nearby subpixel origins should not make retained text jump horizontally: base={full_label:?}, shifted={shifted_full_label:?}"
    );
    assert!(
        shifted_full_label.left.abs_diff(full_label.left) <= 1,
        "nearby subpixel origins should keep the left ink edge stable: base={full_label:?}, shifted={shifted_full_label:?}"
    );
}

#[test]
fn retained_text_measure_uses_runtime_surface_measurement() {
    let style = runtime_single_line_measure_style(10.0, HostTextFontFace::Ui);
    let text = "a\u{0301}b";
    let retained_width = measure_runtime_text_width(text, style.font_size);
    let runtime_width = measure_text_size(text, &style).width;

    assert!((retained_width - runtime_width).abs() < 0.01);
    assert_eq!(measure_runtime_text_width("", style.font_size), 0.0);
    assert_eq!(measure_runtime_text_width(text, 0.0), 0.0);
}

#[test]
fn retained_text_measure_selects_runtime_family_for_ui_and_code_faces() {
    let text = "Preview 123";
    let font_size = 10.0;
    let code_style = UiTextRunPaintStyle {
        code: true,
        ..UiTextRunPaintStyle::default()
    };
    let ui_width = measure_runtime_text_width(text, font_size);
    let mono_width = measure_runtime_text_width_with_style(text, font_size, code_style);
    let ui_style = runtime_single_line_measure_style(font_size, HostTextFontFace::Ui);
    let mono_style = runtime_single_line_measure_style(font_size, HostTextFontFace::Mono);

    assert_eq!(
        ui_style.font_family.as_deref(),
        Some(font_request_for_face(HostTextFontFace::Ui).family.as_str())
    );
    assert_eq!(
        mono_style.font_family.as_deref(),
        Some(
            font_request_for_face(HostTextFontFace::Mono)
                .family
                .as_str()
        )
    );
    assert!((ui_width - measure_text_size(text, &ui_style).width).abs() < 0.01);
    assert!((mono_width - measure_text_size(text, &mono_style).width).abs() < 0.01);
}

#[test]
fn retained_text_measure_carries_runtime_weight_for_each_face() {
    let ui_style = runtime_single_line_measure_style(10.0, HostTextFontFace::Ui);
    let strong_style = runtime_single_line_measure_style(10.0, HostTextFontFace::UiStrong);
    let mono_style = runtime_single_line_measure_style(10.0, HostTextFontFace::Mono);

    assert_eq!(
        ui_style.font_weight,
        font_request_for_face(HostTextFontFace::Ui).weight
    );
    assert_eq!(
        strong_style.font_weight,
        font_request_for_face(HostTextFontFace::UiStrong).weight
    );
    assert_eq!(
        mono_style.font_weight,
        font_request_for_face(HostTextFontFace::Mono).weight
    );
    assert_ne!(
        strong_style.font_weight, ui_style.font_weight,
        "strong UI text must not collapse to the ordinary UI weight"
    );
}

fn runtime_single_line_style(font_size: f32, line_height: f32) -> UiResolvedStyle {
    UiResolvedStyle {
        text_overflow: UiTextOverflow::Ellipsis,
        font_size,
        line_height,
        ..runtime_text_style_for_face(
            HostTextFontFace::Ui,
            font_size,
            line_height,
            UiTextWrap::None,
            UiTextOverflow::Ellipsis,
        )
    }
}

fn runtime_single_line_measure_style(
    font_size: f32,
    font_face: HostTextFontFace,
) -> UiResolvedStyle {
    runtime_text_style_for_face(
        font_face,
        font_size,
        UiResolvedStyle::default_line_height(font_size),
        UiTextWrap::None,
        UiTextOverflow::Clip,
    )
}

fn first_runtime_width_that_keeps_combining_grapheme(
    text: &str,
    style: &UiResolvedStyle,
) -> (f32, String) {
    for width_px in 4..64 {
        let width = width_px as f32;
        let layout = layout_text(text, style, UiFrame::new(0.0, 0.0, width, 24.0), None);
        let Some(line) = layout.lines.first() else {
            continue;
        };
        if line.ellipsized && line.text.starts_with("a\u{0301}") && line.text.ends_with('…') {
            return (width, line.text.clone());
        }
    }

    panic!("runtime text layout did not find a combining-grapheme ellipsis width");
}

#[derive(Debug)]
struct EditorCropInkProfile {
    left: u32,
    painted_pixels: usize,
    ink_center_x: f32,
    max_internal_empty_columns: u32,
}

fn editor_crop_ink_profile(text: &str, x: f32, width: f32) -> EditorCropInkProfile {
    const BACKGROUND: [u8; 4] = [17, 22, 26, 255];
    let mut frame = HostRgbaFrame::filled(180, 36, BACKGROUND);

    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x,
            y: 8.0,
            width,
            height: 22.0,
        },
        text,
        None,
        [224, 232, 238, 255],
        13.0,
        16.0,
        UiTextRunPaintStyle::default(),
    );

    ink_profile_from_frame(&frame, BACKGROUND)
}

fn ink_profile_from_frame(frame: &HostRgbaFrame, background: [u8; 4]) -> EditorCropInkProfile {
    let width = frame.width();
    let height = frame.height();
    let mut left = width;
    let mut right = 0;
    let mut painted_pixels = 0_usize;
    let mut weighted_x = 0_f32;
    let mut columns = vec![false; width as usize];

    for y in 0..height {
        for x in 0..width {
            let offset = ((y as usize * width as usize) + x as usize) * 4;
            let pixel = &frame.as_bytes()[offset..offset + 4];
            if pixel != background {
                left = left.min(x);
                right = right.max(x);
                painted_pixels += 1;
                weighted_x += x as f32;
                columns[x as usize] = true;
            }
        }
    }

    assert!(painted_pixels > 0, "expected visible retained text ink");
    EditorCropInkProfile {
        left,
        painted_pixels,
        ink_center_x: weighted_x / painted_pixels as f32,
        max_internal_empty_columns: max_internal_empty_columns(&columns, left, right),
    }
}

fn max_internal_empty_columns(columns: &[bool], left: u32, right: u32) -> u32 {
    let mut max_run = 0_u32;
    let mut current_run = 0_u32;
    let mut seen_ink = false;
    for column in left..=right {
        if columns[column as usize] {
            seen_ink = true;
            max_run = max_run.max(current_run);
            current_run = 0;
        } else if seen_ink {
            current_run += 1;
        }
    }
    max_run
}
