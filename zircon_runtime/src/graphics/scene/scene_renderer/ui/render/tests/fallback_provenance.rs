use super::*;

#[test]
fn screen_space_ui_plan_routes_virtual_resolved_text_to_sdf_for_exact_advances() {
    let plan = plan_screen_space_ui_batches(
        &text_extract(
            UiResolvedStyle {
                foreground_color: Some("#ffffff".to_string()),
                font_size: 12.0,
                line_height: 16.0,
                text_align: UiTextAlign::Justify,
                text_direction: UiTextDirection::RightToLeft,
                text_render_mode: UiTextRenderMode::Native,
                ..UiResolvedStyle::default()
            },
            UiResolvedTextLayout {
                text_align: UiTextAlign::Justify,
                wrap: UiTextWrap::None,
                direction: UiTextDirection::RightToLeft,
                writing_mode: UiTextWritingMode::HorizontalTb,
                overflow: UiTextOverflow::Clip,
                font_size: 12.0,
                line_height: 16.0,
                measured_width: 80.0,
                measured_height: 16.0,
                source_range: UiTextRange { start: 0, end: 8 },
                lines: vec![UiResolvedTextLine {
                    text: "\u{0633}\u{0640}\u{0644}\u{0627}\u{0645}".to_string(),
                    frame: UiFrame::new(10.0, 20.0, 80.0, 16.0),
                    source_range: UiTextRange { start: 0, end: 8 },
                    visual_range: UiTextRange { start: 0, end: 10 },
                    measured_width: 80.0,
                    glyph_advances: vec![16.0; 5],
                    baseline: 12.0,
                    direction: UiTextDirection::RightToLeft,
                    runs: vec![UiResolvedTextRun {
                        kind: UiTextRunKind::Plain,
                        text: "\u{0640}".to_string(),
                        source_range: UiTextRange { start: 2, end: 2 },
                        visual_range: UiTextRange { start: 2, end: 4 },
                        direction: UiTextDirection::RightToLeft,
                    }],
                    ellipsized: false,
                }],
                boxes: Vec::new(),
                overflow_clipped: false,
                editable: None,
                rich_text_artifact: None,
            },
            "\u{0633}\u{0644}\u{0627}\u{0645}",
            27,
        ),
        UVec2::new(160, 64),
    );

    assert!(plan.native_texts.is_empty());
    assert_eq!(plan.sdf_texts.len(), 1);
    assert_eq!(
        plan.sdf_texts[0].text,
        "\u{0633}\u{0640}\u{0644}\u{0627}\u{0645}"
    );
    assert_eq!(plan.sdf_texts[0].glyph_advances, vec![16.0; 5]);
    assert!(!plan.sdf_texts[0].is_source_isomorphic_layout_line);
}

#[test]
fn screen_space_ui_plan_marks_source_isomorphic_wrapped_layout_lines_for_overlays() {
    let plan = plan_screen_space_ui_batches(
        &text_extract(
            UiResolvedStyle {
                foreground_color: Some("#ffffff".to_string()),
                font_size: 12.0,
                line_height: 16.0,
                wrap: UiTextWrap::Word,
                text_render_mode: UiTextRenderMode::Sdf,
                rich_text_format: UiRichTextFormat::Markdown,
                ..UiResolvedStyle::default()
            },
            UiResolvedTextLayout {
                text_align: UiTextAlign::Left,
                wrap: UiTextWrap::Word,
                direction: UiTextDirection::LeftToRight,
                writing_mode: UiTextWritingMode::HorizontalTb,
                overflow: UiTextOverflow::Clip,
                font_size: 12.0,
                line_height: 16.0,
                measured_width: 40.0,
                measured_height: 16.0,
                source_range: UiTextRange { start: 4, end: 8 },
                lines: vec![UiResolvedTextLine {
                    text: "line".to_string(),
                    frame: UiFrame::new(10.0, 20.0, 40.0, 16.0),
                    source_range: UiTextRange { start: 4, end: 8 },
                    visual_range: UiTextRange { start: 0, end: 4 },
                    measured_width: 40.0,
                    glyph_advances: vec![10.0; 4],
                    baseline: 12.0,
                    direction: UiTextDirection::LeftToRight,
                    runs: vec![UiResolvedTextRun {
                        kind: UiTextRunKind::Plain,
                        text: "line".to_string(),
                        source_range: UiTextRange { start: 4, end: 8 },
                        visual_range: UiTextRange { start: 0, end: 4 },
                        direction: UiTextDirection::LeftToRight,
                    }],
                    ellipsized: false,
                }],
                boxes: Vec::new(),
                overflow_clipped: false,
                editable: None,
                rich_text_artifact: None,
            },
            "pre line",
            28,
        ),
        UVec2::new(160, 64),
    );

    assert!(plan.native_texts.is_empty());
    assert_eq!(plan.sdf_texts.len(), 1);
    assert_eq!(plan.sdf_texts[0].text, "line");
    assert_eq!(plan.sdf_texts[0].wrap, UiTextWrap::Word);
    assert!(plan.sdf_texts[0].is_source_isomorphic_layout_line);
}

#[test]
fn screen_space_ui_plan_uses_resolved_layout_metadata_for_source_isomorphic_paint_runs() {
    let plan = plan_screen_space_ui_batches(
        &text_extract(
            UiResolvedStyle {
                foreground_color: Some("#ffffff".to_string()),
                font_size: 12.0,
                line_height: 16.0,
                text_align: UiTextAlign::Left,
                text_writing_mode: UiTextWritingMode::HorizontalTb,
                wrap: UiTextWrap::None,
                text_render_mode: UiTextRenderMode::Sdf,
                rich_text_format: UiRichTextFormat::Markdown,
                ..UiResolvedStyle::default()
            },
            UiResolvedTextLayout {
                text_align: UiTextAlign::Justify,
                wrap: UiTextWrap::Word,
                direction: UiTextDirection::LeftToRight,
                writing_mode: UiTextWritingMode::VerticalRl,
                overflow: UiTextOverflow::Clip,
                font_size: 12.0,
                line_height: 16.0,
                measured_width: 16.0,
                measured_height: 40.0,
                source_range: UiTextRange { start: 0, end: 4 },
                lines: vec![UiResolvedTextLine {
                    text: "line".to_string(),
                    frame: UiFrame::new(10.0, 20.0, 16.0, 40.0),
                    source_range: UiTextRange { start: 0, end: 4 },
                    visual_range: UiTextRange { start: 0, end: 4 },
                    measured_width: 40.0,
                    glyph_advances: vec![10.0; 4],
                    baseline: 8.0,
                    direction: UiTextDirection::LeftToRight,
                    runs: vec![UiResolvedTextRun {
                        kind: UiTextRunKind::Plain,
                        text: "line".to_string(),
                        source_range: UiTextRange { start: 0, end: 4 },
                        visual_range: UiTextRange { start: 0, end: 4 },
                        direction: UiTextDirection::LeftToRight,
                    }],
                    ellipsized: false,
                }],
                boxes: Vec::new(),
                overflow_clipped: false,
                editable: None,
                rich_text_artifact: None,
            },
            "line",
            29,
        ),
        UVec2::new(160, 96),
    );

    assert!(plan.native_texts.is_empty());
    assert_eq!(plan.sdf_texts.len(), 1);
    assert!(plan.sdf_texts[0].is_source_isomorphic_layout_line);
    assert_eq!(
        plan.sdf_texts[0].writing_mode,
        UiTextWritingMode::VerticalRl
    );
    assert_eq!(plan.sdf_texts[0].text_align, UiTextAlign::Justify);
    assert_eq!(plan.sdf_texts[0].wrap, UiTextWrap::Word);
}

#[test]
fn screen_space_ui_plan_rejects_same_length_visual_text_as_overlay_provenance() {
    let plan = plan_screen_space_ui_batches(
        &text_extract(
            UiResolvedStyle {
                foreground_color: Some("#ffffff".to_string()),
                font_size: 12.0,
                line_height: 16.0,
                wrap: UiTextWrap::Word,
                text_render_mode: UiTextRenderMode::Sdf,
                rich_text_format: UiRichTextFormat::Markdown,
                ..UiResolvedStyle::default()
            },
            UiResolvedTextLayout {
                text_align: UiTextAlign::Left,
                wrap: UiTextWrap::Word,
                direction: UiTextDirection::LeftToRight,
                writing_mode: UiTextWritingMode::HorizontalTb,
                overflow: UiTextOverflow::Clip,
                font_size: 12.0,
                line_height: 16.0,
                measured_width: 40.0,
                measured_height: 16.0,
                source_range: UiTextRange { start: 0, end: 4 },
                lines: vec![UiResolvedTextLine {
                    text: "\u{0640}\u{0640}".to_string(),
                    frame: UiFrame::new(10.0, 20.0, 40.0, 16.0),
                    source_range: UiTextRange { start: 0, end: 4 },
                    visual_range: UiTextRange { start: 0, end: 4 },
                    measured_width: 40.0,
                    glyph_advances: vec![20.0; 2],
                    baseline: 12.0,
                    direction: UiTextDirection::LeftToRight,
                    runs: vec![UiResolvedTextRun {
                        kind: UiTextRunKind::Plain,
                        text: "\u{0640}\u{0640}".to_string(),
                        source_range: UiTextRange { start: 0, end: 4 },
                        visual_range: UiTextRange { start: 0, end: 4 },
                        direction: UiTextDirection::LeftToRight,
                    }],
                    ellipsized: false,
                }],
                boxes: Vec::new(),
                overflow_clipped: false,
                editable: None,
                rich_text_artifact: None,
            },
            "base",
            29,
        ),
        UVec2::new(160, 64),
    );

    assert!(plan.native_texts.is_empty());
    assert_eq!(plan.sdf_texts.len(), 1);
    assert!(!plan.sdf_texts[0].is_source_isomorphic_layout_line);
}

#[test]
fn screen_space_ui_plan_keeps_split_paint_runs_out_of_wrapped_overlay_provenance() {
    let plan = plan_screen_space_ui_batches(
        &text_extract(
            UiResolvedStyle {
                foreground_color: Some("#ffffff".to_string()),
                font_size: 12.0,
                line_height: 16.0,
                wrap: UiTextWrap::Word,
                text_render_mode: UiTextRenderMode::Sdf,
                rich_text_format: UiRichTextFormat::Markdown,
                ..UiResolvedStyle::default()
            },
            UiResolvedTextLayout {
                text_align: UiTextAlign::Left,
                wrap: UiTextWrap::Word,
                direction: UiTextDirection::LeftToRight,
                writing_mode: UiTextWritingMode::HorizontalTb,
                overflow: UiTextOverflow::Clip,
                font_size: 12.0,
                line_height: 16.0,
                measured_width: 40.0,
                measured_height: 16.0,
                source_range: UiTextRange { start: 0, end: 5 },
                lines: vec![UiResolvedTextLine {
                    text: "split".to_string(),
                    frame: UiFrame::new(10.0, 20.0, 40.0, 16.0),
                    source_range: UiTextRange { start: 0, end: 5 },
                    visual_range: UiTextRange { start: 0, end: 5 },
                    measured_width: 40.0,
                    glyph_advances: vec![8.0; 5],
                    baseline: 12.0,
                    direction: UiTextDirection::LeftToRight,
                    runs: vec![
                        UiResolvedTextRun {
                            kind: UiTextRunKind::Plain,
                            text: "sp".to_string(),
                            source_range: UiTextRange { start: 0, end: 2 },
                            visual_range: UiTextRange { start: 0, end: 2 },
                            direction: UiTextDirection::LeftToRight,
                        },
                        UiResolvedTextRun {
                            kind: UiTextRunKind::Strong,
                            text: "lit".to_string(),
                            source_range: UiTextRange { start: 2, end: 5 },
                            visual_range: UiTextRange { start: 2, end: 5 },
                            direction: UiTextDirection::LeftToRight,
                        },
                    ],
                    ellipsized: false,
                }],
                boxes: Vec::new(),
                overflow_clipped: false,
                editable: None,
                rich_text_artifact: None,
            },
            "split",
            30,
        ),
        UVec2::new(160, 64),
    );

    assert!(plan.native_texts.is_empty());
    assert_eq!(plan.sdf_texts.len(), 2);
    assert!(plan
        .sdf_texts
        .iter()
        .all(|text| !text.is_source_isomorphic_layout_line));
    assert!(plan
        .sdf_texts
        .iter()
        .all(|text| matches!(text.wrap, UiTextWrap::None)));
}

fn text_extract(
    style: UiResolvedStyle,
    layout: UiResolvedTextLayout,
    text: &str,
    node_id: u64,
) -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("runtime.ui.fallback-provenance"),
        list: UiRenderList {
            commands: vec![UiRenderCommand {
                node_id: UiNodeId::new(node_id),
                kind: UiRenderCommandKind::Text,
                frame: UiFrame::new(10.0, 20.0, 80.0, 16.0),
                clip_frame: None,
                z_index: 0,
                style,
                text_layout: Some(layout),
                text: Some(text.to_string()),
                image: None,
                opacity: 1.0,
            }],
        },
        raster_scale: 1.0,
    }
}
