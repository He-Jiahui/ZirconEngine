use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::{UiTextAlign, UiTextWrap},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

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
    assert_eq!(layout.lines[1].text, "Gamma Delta");
    assert_eq!(layout.lines[1].frame, UiFrame::new(12.5, 32.0, 55.0, 12.0));
    assert!(!layout.overflow_clipped);
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

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
