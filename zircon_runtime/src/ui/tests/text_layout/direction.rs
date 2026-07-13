use super::*;

#[test]
fn render_extract_preserves_logical_start_text_align() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 80.0, 16.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("LogicalAlignLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "שלום"
font_size = 10.0
line_height = 12.0
text_align = "start"
text_direction = "right_to_left"
wrap = "none"
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
                ..Default::default()
            }),
    );

    surface.rebuild();

    let layout = first_text_layout(&surface);
    assert_eq!(layout.text_align, UiTextAlign::Start);
    assert_eq!(layout.direction, UiTextDirection::RightToLeft);
    assert!((layout.lines[0].frame.right() - 80.0).abs() < 0.01);
}

#[test]
fn render_extract_auto_direction_uses_first_strong_for_logical_start_align() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 16.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("AutoDirectionLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "שלום abc"
font_size = 10.0
line_height = 12.0
text_align = "start"
text_direction = "auto"
wrap = "none"
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
                ..Default::default()
            }),
    );

    surface.rebuild();

    let layout = first_text_layout(&surface);
    assert_eq!(layout.text_align, UiTextAlign::Start);
    assert_eq!(layout.direction, UiTextDirection::RightToLeft);
    assert_eq!(layout.lines[0].direction, UiTextDirection::RightToLeft);
    assert!((layout.lines[0].frame.right() - 120.0).abs() < 0.01);
}

#[test]
fn render_extract_outputs_rich_directional_ellipsis_layout() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 50.0, 12.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("RichLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "Alpha **Beta** שלום Gamma"
font_size = 10.0
line_height = 12.0
wrap = "word"
text_overflow = "ellipsis"
rich_text_format = "markdown"
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
                ..Default::default()
            }),
    );

    surface.rebuild();

    let layout = surface.render_extract.list.commands[0]
        .text_layout
        .as_ref()
        .unwrap();
    assert_eq!(layout.direction, UiTextDirection::LeftToRight);
    assert_eq!(layout.overflow, UiTextOverflow::Ellipsis);
    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert!(layout.lines[0].text.ends_with('…'));
    assert!(layout.lines[0]
        .runs
        .iter()
        .any(|run| run.kind == UiTextRunKind::Strong));
}

#[test]
fn render_extract_outputs_visual_order_ranges_for_mixed_direction_text() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 16.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("MixedDirectionLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "abc שלום def"
font_size = 10.0
line_height = 12.0
wrap = "none"
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
                ..Default::default()
            }),
    );

    surface.rebuild();

    let layout = surface.render_extract.list.commands[0]
        .text_layout
        .as_ref()
        .unwrap();
    let line = &layout.lines[0];
    assert_eq!(layout.direction, UiTextDirection::LeftToRight);
    assert_eq!(line.direction, UiTextDirection::LeftToRight);
    assert_eq!(line.text, "abc םולש def");
    assert_eq!(line.source_range, UiTextRange { start: 0, end: 16 });
    assert_eq!(line.visual_range, UiTextRange { start: 0, end: 16 });
    assert_eq!(line.runs.len(), 3);
    assert_eq!(line.runs[0].text, "abc ");
    assert_eq!(line.runs[0].source_range, UiTextRange { start: 0, end: 4 });
    assert_eq!(line.runs[0].visual_range, UiTextRange { start: 0, end: 4 });
    assert_eq!(line.runs[0].direction, UiTextDirection::LeftToRight);
    assert_eq!(line.runs[1].text, "םולש");
    assert_eq!(line.runs[1].source_range, UiTextRange { start: 4, end: 12 });
    assert_eq!(line.runs[1].visual_range, UiTextRange { start: 4, end: 12 });
    assert_eq!(line.runs[1].direction, UiTextDirection::RightToLeft);
    assert_eq!(line.runs[2].text, " def");
    assert_eq!(
        line.runs[2].source_range,
        UiTextRange { start: 12, end: 16 }
    );
    assert_eq!(
        line.runs[2].visual_range,
        UiTextRange { start: 12, end: 16 }
    );
    assert_eq!(line.runs[2].direction, UiTextDirection::LeftToRight);
}

#[test]
fn render_extract_keeps_neutral_separator_inside_rtl_visual_span() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 16.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("MixedDirectionNeutralLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "abc שלום-עולם def"
font_size = 10.0
line_height = 12.0
wrap = "none"
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
                ..Default::default()
            }),
    );

    surface.rebuild();

    let layout = surface.render_extract.list.commands[0]
        .text_layout
        .as_ref()
        .unwrap();
    let line = &layout.lines[0];
    assert_eq!(layout.direction, UiTextDirection::LeftToRight);
    assert_eq!(line.text, "abc םלוע-םולש def");
    assert_eq!(line.source_range, UiTextRange { start: 0, end: 25 });
    assert_eq!(line.visual_range, UiTextRange { start: 0, end: 25 });
    assert_eq!(line.runs.len(), 5);
    assert_eq!(line.runs[0].text, "abc ");
    assert_eq!(line.runs[0].source_range, UiTextRange { start: 0, end: 4 });
    assert_eq!(line.runs[0].visual_range, UiTextRange { start: 0, end: 4 });
    assert_eq!(line.runs[0].direction, UiTextDirection::LeftToRight);
    assert_eq!(line.runs[1].text, "םלוע");
    assert_eq!(
        line.runs[1].source_range,
        UiTextRange { start: 13, end: 21 }
    );
    assert_eq!(line.runs[1].visual_range, UiTextRange { start: 4, end: 12 });
    assert_eq!(line.runs[1].direction, UiTextDirection::RightToLeft);
    assert_eq!(line.runs[2].text, "-");
    assert_eq!(
        line.runs[2].source_range,
        UiTextRange { start: 12, end: 13 }
    );
    assert_eq!(
        line.runs[2].visual_range,
        UiTextRange { start: 12, end: 13 }
    );
    assert_eq!(line.runs[2].direction, UiTextDirection::RightToLeft);
    assert_eq!(line.runs[3].text, "םולש");
    assert_eq!(line.runs[3].source_range, UiTextRange { start: 4, end: 12 });
    assert_eq!(
        line.runs[3].visual_range,
        UiTextRange { start: 13, end: 21 }
    );
    assert_eq!(line.runs[3].direction, UiTextDirection::RightToLeft);
    assert_eq!(line.runs[4].text, " def");
    assert_eq!(
        line.runs[4].source_range,
        UiTextRange { start: 21, end: 25 }
    );
    assert_eq!(
        line.runs[4].visual_range,
        UiTextRange { start: 21, end: 25 }
    );
    assert_eq!(line.runs[4].direction, UiTextDirection::LeftToRight);
}
