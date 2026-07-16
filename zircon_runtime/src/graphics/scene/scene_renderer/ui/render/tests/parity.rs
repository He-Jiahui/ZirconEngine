use super::*;
use crate::text::raster::{raster_path_for, GlyphRasterPath};
use crate::text::sdf::SdfMode;

const BITMAP_SIDE_SIZE: f32 = 23.5;
const SDF_SIDE_SIZE: f32 = 24.0;

#[test]
fn text_paragraph_parity_native_vs_sdf_bbox_advance_linebreak() {
    let cases = [
        (
            "latin",
            "Alpha beta gamma delta",
            UiTextDirection::LeftToRight,
            "en",
        ),
        (
            "cjk",
            "中文段落换行一致验证",
            UiTextDirection::LeftToRight,
            "zh-Hans",
        ),
        (
            "mixed",
            "Build 构建 parity 2026",
            UiTextDirection::LeftToRight,
            "zh-Hans",
        ),
        (
            "rtl",
            "مرحبا بالعالم 2026",
            UiTextDirection::RightToLeft,
            "ar",
        ),
    ];

    assert_eq!(
        raster_path_for(BITMAP_SIDE_SIZE, false),
        GlyphRasterPath::Bitmap
    );
    assert_eq!(raster_path_for(SDF_SIDE_SIZE, false), GlyphRasterPath::Sdf);

    for (label, text, direction, language) in cases {
        for font_size in [BITMAP_SIDE_SIZE, SDF_SIDE_SIZE] {
            let frame = UiFrame::new(12.0, 18.0, 104.0, 220.0);
            let base_style = UiResolvedStyle {
                foreground_color: Some("#f5f7fb".to_string()),
                font_size,
                line_height: font_size + 6.0,
                wrap: UiTextWrap::Glyph,
                text_align: if matches!(direction, UiTextDirection::RightToLeft) {
                    UiTextAlign::Right
                } else {
                    UiTextAlign::Left
                },
                text_direction: direction,
                language: Some(language.to_string()),
                ..UiResolvedStyle::default()
            };

            let (native_style, native_layout) =
                layout_for_mode(text, frame, &base_style, UiTextRenderMode::Native);
            assert!(
                native_layout.lines.len() > 1,
                "{label} {font_size}px fixture must exercise line breaking"
            );
            assert!(
                native_layout
                    .lines
                    .iter()
                    .all(|line| !line.glyph_advances.is_empty()),
                "{label} {font_size}px must retain resolved advances"
            );

            for (distance_mode, expected_sdf_mode) in [
                (UiTextRenderMode::Sdf, SdfMode::Sdf),
                (UiTextRenderMode::Msdf, SdfMode::Msdf),
            ] {
                let (distance_style, distance_layout) =
                    layout_for_mode(text, frame, &base_style, distance_mode);
                assert_layout_parity(label, &native_layout, &distance_layout);

                let native = batches_for_mode(text, frame, native_style.clone(), &native_layout);
                let distance = batches_for_mode(text, frame, distance_style, &distance_layout);
                assert_batch_parity(
                    label,
                    &native_layout,
                    &distance_layout,
                    &native,
                    &distance,
                    expected_sdf_mode,
                );
            }
        }
    }
}

#[test]
fn text_paragraph_parity_vertical_rl() {
    let text = "竖排Native与SDF一致验证";
    let frame = UiFrame::new(12.0, 18.0, 96.0, 72.0);
    for font_size in [BITMAP_SIDE_SIZE, SDF_SIDE_SIZE] {
        let base_style = UiResolvedStyle {
            foreground_color: Some("#f5f7fb".to_string()),
            font_size,
            line_height: font_size + 6.0,
            wrap: UiTextWrap::Glyph,
            text_align: UiTextAlign::Left,
            text_writing_mode: UiTextWritingMode::VerticalRl,
            language: Some("zh-Hans".to_string()),
            ..UiResolvedStyle::default()
        };
        let (native_style, native_layout) =
            layout_for_mode(text, frame, &base_style, UiTextRenderMode::Native);
        assert!(native_layout.lines.len() > 1);
        assert_eq!(native_layout.writing_mode, UiTextWritingMode::VerticalRl);
        assert!(native_layout
            .lines
            .windows(2)
            .all(|columns| columns[0].frame.x > columns[1].frame.x));

        for (distance_mode, expected_sdf_mode) in [
            (UiTextRenderMode::Sdf, SdfMode::Sdf),
            (UiTextRenderMode::Msdf, SdfMode::Msdf),
        ] {
            let (distance_style, distance_layout) =
                layout_for_mode(text, frame, &base_style, distance_mode);
            assert_layout_parity("vertical-rl", &native_layout, &distance_layout);

            let native = batches_for_mode(text, frame, native_style.clone(), &native_layout);
            let distance = batches_for_mode(text, frame, distance_style, &distance_layout);
            assert_batch_parity(
                "vertical-rl",
                &native_layout,
                &distance_layout,
                &native,
                &distance,
                expected_sdf_mode,
            );
        }
    }
}

fn layout_for_mode(
    text: &str,
    frame: UiFrame,
    base_style: &UiResolvedStyle,
    render_mode: UiTextRenderMode,
) -> (UiResolvedStyle, UiResolvedTextLayout) {
    let mut style = base_style.clone();
    style.text_render_mode = render_mode;
    let layout = layout_text(text, &style, frame, None);
    (style, layout)
}

fn assert_layout_parity(
    label: &str,
    native: &UiResolvedTextLayout,
    distance: &UiResolvedTextLayout,
) {
    assert_eq!(native.source_range, distance.source_range, "{label} range");
    assert_eq!(native.lines.len(), distance.lines.len(), "{label} lines");
    assert_f32_bits(
        label,
        "paragraph width",
        native.measured_width,
        distance.measured_width,
    );
    assert_f32_bits(
        label,
        "paragraph height",
        native.measured_height,
        distance.measured_height,
    );

    for (line_index, (native_line, distance_line)) in
        native.lines.iter().zip(&distance.lines).enumerate()
    {
        assert_eq!(
            native_line.text, distance_line.text,
            "{label} line {line_index} text"
        );
        assert_eq!(
            native_line.source_range, distance_line.source_range,
            "{label} line {line_index} byte range"
        );
        assert_eq!(
            native_line.glyph_advances.len(),
            distance_line.glyph_advances.len(),
            "{label} line {line_index} advance count"
        );
        assert_frame_bits(label, line_index, native_line.frame, distance_line.frame);
        for (glyph_index, (native_advance, distance_advance)) in native_line
            .glyph_advances
            .iter()
            .zip(&distance_line.glyph_advances)
            .enumerate()
        {
            assert_eq!(
                native_advance.to_bits(),
                distance_advance.to_bits(),
                "{label} line {line_index} glyph {glyph_index} advance"
            );
        }
    }
}

fn assert_batch_parity(
    label: &str,
    native_layout: &UiResolvedTextLayout,
    distance_layout: &UiResolvedTextLayout,
    native: &[ScreenSpaceUiTextBatch],
    distance: &[ScreenSpaceUiTextBatch],
    expected_sdf_mode: SdfMode,
) {
    assert_eq!(
        native.len(),
        native_layout.lines.len(),
        "{label} native lines"
    );
    assert_eq!(
        distance.len(),
        distance_layout.lines.len(),
        "{label} distance-field lines"
    );
    for (((native_line, distance_line), native_layout_line), distance_layout_line) in native
        .iter()
        .zip(distance)
        .zip(&native_layout.lines)
        .zip(&distance_layout.lines)
    {
        assert_eq!(native_line.text, distance_line.text, "{label} text");
        assert_eq!(
            native_line.text, native_layout_line.text,
            "{label} native layout text"
        );
        assert_eq!(
            distance_line.text, distance_layout_line.text,
            "{label} distance layout text"
        );
        assert_eq!(
            native_line.source_range,
            Some(native_layout_line.source_range),
            "{label} native source range"
        );
        assert_eq!(
            distance_line.source_range,
            Some(distance_layout_line.source_range),
            "{label} distance-field source range"
        );
        assert_eq!(native_line.writing_mode, native_layout.writing_mode);
        assert_eq!(distance_line.writing_mode, distance_layout.writing_mode);
        assert_eq!(distance_line.distance_field_mode, expected_sdf_mode);
        assert_frame_bits(label, 0, native_line.frame, native_layout_line.frame);
        assert_frame_bits(label, 0, distance_line.frame, distance_layout_line.frame);
        assert_advance_bits(
            label,
            &native_line.glyph_advances,
            &native_layout_line.glyph_advances,
        );
        assert_advance_bits(
            label,
            &distance_line.glyph_advances,
            &distance_layout_line.glyph_advances,
        );
        assert_advance_bits(
            label,
            &native_line.glyph_advances,
            &distance_line.glyph_advances,
        );
    }
}

fn batches_for_mode(
    text: &str,
    frame: UiFrame,
    style: UiResolvedStyle,
    layout: &UiResolvedTextLayout,
) -> Vec<ScreenSpaceUiTextBatch> {
    let render_mode = style.text_render_mode;
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.text.parity.vertical"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(8),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout.clone()),
                    text: Some(text.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(180, 180),
    );

    match render_mode {
        UiTextRenderMode::Native => plan.native_texts,
        UiTextRenderMode::Sdf | UiTextRenderMode::Msdf | UiTextRenderMode::Mtsdf => plan.sdf_texts,
        UiTextRenderMode::Auto => plan.auto_texts,
    }
}

fn assert_frame_bits(label: &str, line_index: usize, actual: UiFrame, expected: UiFrame) {
    for (field, actual, expected) in [
        ("x", actual.x, expected.x),
        ("y", actual.y, expected.y),
        ("width", actual.width, expected.width),
        ("height", actual.height, expected.height),
    ] {
        assert_f32_bits(
            label,
            &format!("line {line_index} frame {field}"),
            actual,
            expected,
        );
    }
}

fn assert_advance_bits(label: &str, actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len(), "{label} advance count");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{label} glyph {index} advance"
        );
    }
}

fn assert_f32_bits(label: &str, field: &str, actual: f32, expected: f32) {
    assert_eq!(
        actual.to_bits(),
        expected.to_bits(),
        "{label} {field}: actual={actual}, expected={expected}"
    );
}
