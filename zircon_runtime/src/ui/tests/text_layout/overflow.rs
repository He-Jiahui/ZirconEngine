use super::*;

#[test]
fn render_extract_parses_start_ellipsis_overflow() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 24.0, 12.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("StartEllipsisLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "abcdef"
font_size = 10.0
line_height = 12.0
wrap = "glyph"
text_overflow = "ellipsis_start"
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
    assert_eq!(layout.overflow, UiTextOverflow::EllipsisStart);
    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert!(layout.lines[0].text.starts_with('…'));
    assert!(layout.lines[0].text.ends_with('f'));
}

#[test]
fn render_extract_parses_word_ellipsis_overflow() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 48.0, 12.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("WordEllipsisLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "alpha beta gamma"
font_size = 10.0
line_height = 12.0
wrap = "glyph"
text_overflow = "trim_word_ellipsis"
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
    assert_eq!(layout.overflow, UiTextOverflow::EllipsisWord);
    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert!(layout.lines[0].text.ends_with('…'));
    assert!(!layout.lines[0].text.contains(" b"));
}

#[test]
fn render_extract_parses_middle_ellipsis_overflow() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 24.0, 12.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("MiddleEllipsisLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "abcdef"
font_size = 10.0
line_height = 12.0
wrap = "glyph"
text_overflow = "ellipsis_middle"
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
    assert_eq!(layout.overflow, UiTextOverflow::EllipsisMiddle);
    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert!(layout.lines[0].text.starts_with('a'));
    assert!(layout.lines[0].text.contains('…'));
    assert!(layout.lines[0].text.ends_with('f'));
}

#[test]
fn render_extract_parses_shrink_to_fit_overflow_and_scales_font() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 60.0, 24.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("ShrinkToFitLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "Wide runtime text"
font_size = 20.0
line_height = 24.0
wrap = "none"
text_overflow = "shrink_to_fit"
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
    assert_eq!(layout.overflow, UiTextOverflow::ShrinkToFit);
    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "Wide runtime text");
    assert!(!layout.lines[0].ellipsized);
    assert!(layout.font_size < 20.0);
    assert!(layout.line_height < 24.0);
    assert!(layout.measured_width <= 60.5);
}

#[test]
fn render_extract_parses_clamp_font_size_overflow_and_bounds() {
    let text = "Clamp runtime text";
    let frame_width = clamp_minimum_fitting_width(text);
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, frame_width, 24.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("ClampFontSizeLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "Clamp runtime text"
font_size = 24.0
line_height = 30.0
wrap = "none"
text_overflow = "clamp_font_size"
min_font_size = 8.0
max_font_size = 18.0
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
    assert_eq!(
        layout.overflow,
        UiTextOverflow::ClampFontSize {
            min_px: 8.0,
            max_px: 18.0,
        }
    );
    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, text);
    assert!(!layout.lines[0].ellipsized);
    assert!(layout.font_size <= 18.0);
    assert!(layout.font_size >= 8.0);
    assert!(layout.measured_width <= frame_width + 0.5);
}

#[test]
fn render_extract_clips_text_layout_to_clip_frame() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 40.0, 48.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("ClippedLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "Alpha Beta Gamma"
font_size = 10.0
line_height = 12.0
wrap = "glyph"
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
    surface
        .tree
        .node_mut(UiNodeId::new(1))
        .unwrap()
        .layout_cache
        .clip_frame = Some(UiFrame::new(0.0, 0.0, 40.0, 12.0));

    surface.rebuild();

    let command = surface.render_extract.list.commands.first().unwrap();
    let layout = command.text_layout.as_ref().unwrap();
    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "Alpha Be");
    assert!(layout.overflow_clipped);
}

fn clamp_minimum_fitting_width(text: &str) -> f32 {
    let min_style = UiResolvedStyle {
        font_size: 8.0,
        line_height: 10.0,
        wrap: UiTextWrap::None,
        ..UiResolvedStyle::default()
    };
    measure_text_size(text, &min_style).width + 0.25
}
