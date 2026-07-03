use super::*;

#[test]
fn render_extract_outputs_aligned_wrapped_text_layout() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(10.0, 20.0, 60.0, 48.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("RuntimeLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "Alpha Beta Gamma Delta"
font_size = 10.0
line_height = 12.0
text_align = "center"
wrap = "word"
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

    let command = surface.render_extract.list.commands.first().unwrap();
    let layout = command
        .text_layout
        .as_ref()
        .expect("text command should carry resolved layout data");
    assert_eq!(layout.text_align, UiTextAlign::Center);
    assert_eq!(layout.wrap, UiTextWrap::Word);
    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "Alpha Beta");
    assert_eq!(layout.lines[0].frame, UiFrame::new(15.0, 20.0, 50.0, 12.0));
    assert_eq!(layout.lines[0].source_range.start, 0);
    assert_eq!(layout.lines[0].runs[0].kind, UiTextRunKind::Plain);
    assert_eq!(layout.lines[1].text, "Gamma Delta");
    assert_eq!(layout.lines[1].frame, UiFrame::new(12.5, 32.0, 55.0, 12.0));
    assert!(!layout.overflow_clipped);
}

#[test]
fn render_extract_parses_justify_text_align_and_expands_non_final_line() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 96.0, 24.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("JustifyLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "a b 中文\ntail"
font_size = 10.0
line_height = 12.0
text_align = "justify"
wrap = "word"
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
    assert_eq!(layout.text_align, UiTextAlign::Justify);
    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "a b 中文");
    assert!((layout.lines[0].frame.width - 96.0).abs() < 0.1);
    assert!((layout.lines[0].glyph_advances.iter().sum::<f32>() - 96.0).abs() < 0.1);
    assert!(layout.lines[1].frame.width < 96.0);
}
