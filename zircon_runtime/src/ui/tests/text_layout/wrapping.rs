use super::*;

#[test]
fn render_extract_parses_word_smart_wrap_layout() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 20.0, 48.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("WordSmartLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "abcd"
font_size = 10.0
line_height = 12.0
wrap = "word_smart"
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
    assert_eq!(layout.wrap, UiTextWrap::WordSmart);
    assert!(layout.lines.len() > 1);
    assert_eq!(
        layout
            .lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<String>(),
        "abcd"
    );
}

#[test]
fn render_extract_parses_word_smart_wrap_alias() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 32.0, 24.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("WordSmartAliasLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "Alpha Beta"
font_size = 10.0
line_height = 12.0
wrap = "word-smart"
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
    assert_eq!(layout.wrap, UiTextWrap::WordSmart);
}

#[test]
fn render_extract_parses_vertical_rl_writing_mode_layout() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let frame_height = measure_text_size("縦書", &style).width + 0.1;
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 36.0, frame_height))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("VerticalWritingLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "縦書文"
font_size = 10.0
line_height = 12.0
wrap = "word"
writing_mode = "vertical-rl"
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

    let command = first_text_layout_command(&surface);
    let layout = command
        .text_layout
        .as_ref()
        .expect("vertical text command should carry resolved layout data");
    assert_eq!(
        command.style.text_writing_mode,
        UiTextWritingMode::VerticalRl
    );
    assert_eq!(layout.writing_mode, UiTextWritingMode::VerticalRl);
    assert_eq!(layout.lines.len(), 2);
    assert!(layout.lines[0].frame.x > layout.lines[1].frame.x);
    assert_eq!(layout.lines[0].frame.y, layout.lines[1].frame.y);
}
