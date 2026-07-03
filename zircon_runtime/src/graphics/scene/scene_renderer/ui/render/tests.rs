use super::*;
use crate::core::math::UVec2;
use crate::ui::text::layout_text;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::surface::{
    UiEditableTextState, UiRenderExtract, UiRenderList, UiResolvedStyle, UiResolvedTextLayout,
    UiResolvedTextLine, UiResolvedTextRun, UiTextAlign, UiTextCaret, UiTextCaretAffinity,
    UiTextComposition, UiTextDirection, UiTextOverflow, UiTextRange, UiTextRenderMode,
    UiTextRunKind, UiTextSelection, UiTextWrap, UiTextWritingMode,
};

#[test]
fn screen_space_ui_plan_keeps_text_batches_for_quad_commands() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(1),
                    kind: UiRenderCommandKind::Quad,
                    frame: UiFrame::new(8.0, 12.0, 120.0, 36.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        background_color: Some("#112233".to_string()),
                        foreground_color: Some("#ddeeff".to_string()),
                        font_weight: 650,
                        font_size: 18.0,
                        line_height: 22.0,
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::Word,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Launch".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(200, 100),
    );

    assert_eq!(plan.draws.len(), 1);
    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.native_texts[0].font_weight, 650);
    assert!(plan.sdf_texts.is_empty());
}

#[test]
fn screen_space_ui_plan_routes_sdf_text_to_a_separate_batch() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(2),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(0.0, 0.0, 160.0, 48.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 20.0,
                        line_height: 24.0,
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::Word,
                        text_render_mode: UiTextRenderMode::Sdf,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("SDF".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(320, 180),
    );

    assert!(plan.native_texts.is_empty());
    assert_eq!(plan.sdf_texts.len(), 1);
}

#[test]
fn screen_space_ui_plan_keeps_auto_text_in_a_separate_batch() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(3),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(4.0, 6.0, 144.0, 40.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font: Some("res://fonts/default.font.toml".to_string()),
                        font_size: 16.0,
                        line_height: 20.0,
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::Word,
                        text_render_mode: UiTextRenderMode::Auto,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Auto".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(320, 180),
    );

    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert_eq!(plan.auto_texts.len(), 1);
}

#[test]
fn screen_space_ui_plan_uses_resolved_text_layout_lines_as_batches() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(4),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 90.0, 48.0),
                    clip_frame: Some(UiFrame::new(0.0, 0.0, 120.0, 48.0)),
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        font_weight: 500,
                        line_height: 12.0,
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::Word,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::Word,
                        direction: UiTextDirection::LeftToRight,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Clip,
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 50.0,
                        measured_height: 24.0,
                        source_range: UiTextRange { start: 0, end: 16 },
                        lines: vec![
                            UiResolvedTextLine {
                                text: "Alpha Beta".to_string(),
                                frame: UiFrame::new(20.0, 20.0, 50.0, 12.0),
                                source_range: UiTextRange { start: 0, end: 10 },
                                visual_range: UiTextRange { start: 0, end: 10 },
                                measured_width: 50.0,
                                glyph_advances: vec![],
                                baseline: 8.0,
                                direction: UiTextDirection::LeftToRight,
                                runs: vec![UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "Alpha Beta".to_string(),
                                    source_range: UiTextRange { start: 0, end: 10 },
                                    visual_range: UiTextRange { start: 0, end: 10 },
                                    direction: UiTextDirection::LeftToRight,
                                }],
                                ellipsized: false,
                            },
                            UiResolvedTextLine {
                                text: "Gamma".to_string(),
                                frame: UiFrame::new(35.0, 32.0, 25.0, 12.0),
                                source_range: UiTextRange { start: 11, end: 16 },
                                visual_range: UiTextRange { start: 0, end: 5 },
                                measured_width: 25.0,
                                glyph_advances: vec![],
                                baseline: 8.0,
                                direction: UiTextDirection::LeftToRight,
                                runs: vec![UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "Gamma".to_string(),
                                    source_range: UiTextRange { start: 11, end: 16 },
                                    visual_range: UiTextRange { start: 0, end: 5 },
                                    direction: UiTextDirection::LeftToRight,
                                }],
                                ellipsized: false,
                            },
                        ],
                        overflow_clipped: false,
                        editable: None,
                    }),
                    text: Some("Alpha Beta Gamma".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(160, 120),
    );

    assert_eq!(plan.native_texts.len(), 2);
    assert_eq!(plan.native_texts[0].text, "Alpha Beta");
    assert_eq!(
        plan.native_texts[0].frame,
        UiFrame::new(20.0, 20.0, 50.0, 12.0)
    );
    assert_eq!(plan.native_texts[1].text, "Gamma");
    assert_eq!(
        plan.native_texts[1].frame,
        UiFrame::new(35.0, 32.0, 25.0, 12.0)
    );
    assert_eq!(
        plan.native_texts[0].clip_frame,
        Some(UiFrame::new(0.0, 0.0, 120.0, 48.0))
    );
    assert_eq!(
        plan.native_texts[0].source_range,
        Some(UiTextRange { start: 0, end: 10 })
    );
    assert_eq!(plan.native_texts[0].wrap, UiTextWrap::None);
    assert_eq!(plan.native_texts[0].text_align, UiTextAlign::Left);
}

#[test]
fn text_paragraph_parity_native_vs_sdf_bbox_advance_linebreak() {
    let text = "Alpha世界Beta Gamma";
    let frame = UiFrame::new(12.0, 18.0, 46.0, 144.0);
    let style = UiResolvedStyle {
        foreground_color: Some("#f5f7fb".to_string()),
        font_size: 14.0,
        line_height: 18.0,
        wrap: UiTextWrap::Glyph,
        text_align: UiTextAlign::Left,
        text_render_mode: UiTextRenderMode::Native,
        ..UiResolvedStyle::default()
    };
    let layout = layout_text(text, &style, frame, None);
    assert!(
        layout.lines.len() > 1,
        "parity fixture must exercise real line breaking"
    );
    assert!(
        layout
            .lines
            .iter()
            .all(|line| !line.glyph_advances.is_empty()),
        "shared layout must provide per-glyph advances for parity"
    );

    let native = text_batches_for_render_mode(text, &style, &layout, UiTextRenderMode::Native);
    let sdf = text_batches_for_render_mode(text, &style, &layout, UiTextRenderMode::Sdf);

    assert_eq!(native.len(), layout.lines.len());
    assert_eq!(sdf.len(), layout.lines.len());
    for ((native_line, sdf_line), layout_line) in native.iter().zip(sdf.iter()).zip(&layout.lines) {
        assert_eq!(native_line.text, sdf_line.text);
        assert_eq!(native_line.text, layout_line.text);
        assert_eq!(native_line.source_range, Some(layout_line.source_range));
        assert_eq!(sdf_line.source_range, Some(layout_line.source_range));
        assert_eq!(native_line.wrap, UiTextWrap::None);
        assert_eq!(sdf_line.wrap, UiTextWrap::None);
        assert_frame_close(native_line.frame, layout_line.frame);
        assert_frame_close(sdf_line.frame, layout_line.frame);
        assert_advance_rows_close(&native_line.glyph_advances, &layout_line.glyph_advances);
        assert_advance_rows_close(&sdf_line.glyph_advances, &layout_line.glyph_advances);
        assert_advance_rows_close(&native_line.glyph_advances, &sdf_line.glyph_advances);
    }
}

#[test]
fn screen_space_ui_plan_splits_rich_text_runs_from_shared_paint() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(6),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 150.0, 18.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        font_weight: 500,
                        line_height: 12.0,
                        text_render_mode: UiTextRenderMode::Native,
                        rich_text: true,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::None,
                        direction: UiTextDirection::LeftToRight,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Clip,
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 150.0,
                        measured_height: 12.0,
                        source_range: UiTextRange { start: 0, end: 15 },
                        lines: vec![UiResolvedTextLine {
                            text: "Alpha Beta Code".to_string(),
                            frame: UiFrame::new(10.0, 20.0, 150.0, 12.0),
                            source_range: UiTextRange { start: 0, end: 15 },
                            visual_range: UiTextRange { start: 0, end: 15 },
                            measured_width: 150.0,
                            glyph_advances: vec![],
                            baseline: 8.0,
                            direction: UiTextDirection::LeftToRight,
                            runs: vec![
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "Alpha ".to_string(),
                                    source_range: UiTextRange { start: 0, end: 6 },
                                    visual_range: UiTextRange { start: 0, end: 6 },
                                    direction: UiTextDirection::LeftToRight,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Strong,
                                    text: "Beta".to_string(),
                                    source_range: UiTextRange { start: 6, end: 10 },
                                    visual_range: UiTextRange { start: 6, end: 10 },
                                    direction: UiTextDirection::LeftToRight,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Code,
                                    text: " Code".to_string(),
                                    source_range: UiTextRange { start: 10, end: 15 },
                                    visual_range: UiTextRange { start: 10, end: 15 },
                                    direction: UiTextDirection::LeftToRight,
                                },
                            ],
                            ellipsized: false,
                        }],
                        overflow_clipped: false,
                        editable: None,
                    }),
                    text: Some("Alpha Beta Code".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(220, 80),
    );

    assert_eq!(plan.native_texts.len(), 3);
    assert_eq!(plan.native_texts[0].text, "Alpha ");
    assert_eq!(plan.native_texts[1].text, "Beta");
    assert_eq!(plan.native_texts[2].text, " Code");
    assert_eq!(
        plan.native_texts[0].frame,
        UiFrame::new(10.0, 20.0, 60.0, 12.0)
    );
    assert_eq!(
        plan.native_texts[1].frame,
        UiFrame::new(70.0, 20.0, 40.0, 12.0)
    );
    assert!(plan.native_texts[1].style.strong);
    assert!(plan.native_texts[2].style.code);
    assert!(plan.native_texts.iter().all(|text| text.font_weight == 500));
}

fn text_batches_for_render_mode(
    text: &str,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
    render_mode: UiTextRenderMode,
) -> Vec<ScreenSpaceUiTextBatch> {
    let mut style = style.clone();
    style.text_render_mode = render_mode;
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.text.parity"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(7),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(12.0, 18.0, 46.0, 144.0),
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
        UiTextRenderMode::Sdf => plan.sdf_texts,
        UiTextRenderMode::Auto => plan.auto_texts,
    }
}

fn assert_frame_close(actual: UiFrame, expected: UiFrame) {
    assert!(
        (actual.x - expected.x).abs() < 0.01
            && (actual.y - expected.y).abs() < 0.01
            && (actual.width - expected.width).abs() < 0.01
            && (actual.height - expected.height).abs() < 0.01,
        "frame mismatch: actual={actual:?} expected={expected:?}"
    );
}

fn assert_advance_rows_close(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (*actual - *expected).abs() < 0.01,
            "advance[{index}] mismatch: actual={actual} expected={expected}"
        );
    }
}

#[test]
fn screen_space_ui_plan_uses_shared_text_decorations_as_pre_and_post_text_draws() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(5),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 50.0, 12.0),
                    clip_frame: Some(UiFrame::new(0.0, 0.0, 80.0, 48.0)),
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        line_height: 12.0,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        direction: UiTextDirection::LeftToRight,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Clip,
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 50.0,
                        measured_height: 12.0,
                        source_range: UiTextRange { start: 0, end: 5 },
                        editable: Some(UiEditableTextState {
                            text: "Hello".to_string(),
                            caret: UiTextCaret {
                                offset: 4,
                                affinity: UiTextCaretAffinity::Downstream,
                            },
                            selection: Some(UiTextSelection {
                                anchor: 1,
                                focus: 3,
                            }),
                            composition: Some(UiTextComposition {
                                range: UiTextRange { start: 2, end: 4 },
                                text: "ll".to_string(),
                                restore_text: None,
                            }),
                            read_only: false,
                        }),
                        lines: vec![UiResolvedTextLine {
                            text: "Hello".to_string(),
                            frame: UiFrame::new(10.0, 20.0, 50.0, 12.0),
                            source_range: UiTextRange { start: 0, end: 5 },
                            visual_range: UiTextRange { start: 0, end: 5 },
                            measured_width: 50.0,
                            glyph_advances: vec![],
                            baseline: 8.0,
                            direction: UiTextDirection::LeftToRight,
                            runs: vec![UiResolvedTextRun {
                                kind: UiTextRunKind::Plain,
                                text: "Hello".to_string(),
                                source_range: UiTextRange { start: 0, end: 5 },
                                visual_range: UiTextRange { start: 0, end: 5 },
                                direction: UiTextDirection::LeftToRight,
                            }],
                            ellipsized: false,
                        }],
                        ..UiResolvedTextLayout::default()
                    }),
                    text: Some("Hello".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(80, 48),
    );

    assert_eq!(plan.draws.len(), 1);
    assert_eq!(plan.draws[0].vertices, 0..6);
    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.post_text_draws.len(), 1);
    assert_eq!(plan.post_text_draws[0].vertices, 6..18);
}
