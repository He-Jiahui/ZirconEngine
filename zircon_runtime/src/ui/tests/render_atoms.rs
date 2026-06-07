use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommandKind, UiTextAlign, UiTextRenderMode, UiVisualAssetRef},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_carries_label_and_icon_atoms_through_generic_path() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.atoms"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 90.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Label",
        UiFrame::new(12.0, 10.0, 132.0, 18.0),
        r##"
text = "Scene Graph"
foreground_color = "#ccd9dd"
font_family = "Inter"
font_size = 13.0
line_height = 16.0
text_align = "left"
text_render_mode = "native"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Icon",
        UiFrame::new(150.0, 10.0, 18.0, 18.0),
        r##"
icon = "layers"
foreground_color = "#35c7d0"
opacity = 0.8
"##,
        visible_state(),
    );
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .layout_cache
        .clip_frame = Some(UiFrame::new(0.0, 0.0, 170.0, 32.0));

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let label = commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .expect("label atom should emit a render command");
    assert_eq!(label.kind, UiRenderCommandKind::Text);
    assert_eq!(label.frame, UiFrame::new(12.0, 10.0, 132.0, 18.0));
    assert_eq!(label.clip_frame, Some(UiFrame::new(0.0, 0.0, 170.0, 32.0)));
    assert_eq!(label.text.as_deref(), Some("Scene Graph"));
    assert_eq!(label.image, None);
    assert_eq!(label.style.painter_family, UiPainterFamily::Generic);
    assert_eq!(label.style.painter_state, UiPainterResolvedState::Normal);
    assert_eq!(label.style.foreground_color.as_deref(), Some("#ccd9dd"));
    assert_eq!(label.style.font_family.as_deref(), Some("Inter"));
    assert_eq!(label.style.font_size, 13.0);
    assert_eq!(label.style.line_height, 16.0);
    assert_eq!(label.style.text_align, UiTextAlign::Left);
    assert_eq!(label.style.text_render_mode, UiTextRenderMode::Native);
    let label_layout = label
        .text_layout
        .as_ref()
        .expect("label atom should carry text layout");
    assert_eq!(label_layout.lines.len(), 1);
    assert_eq!(label_layout.lines[0].text, "Scene Graph");
    assert_eq!(label_layout.lines[0].frame.x, label.frame.x);
    assert_eq!(label_layout.lines[0].frame.y, label.frame.y);
    assert_eq!(label_layout.source_range.start, 0);
    assert_eq!(label_layout.source_range.end, "Scene Graph".len());

    let icon = commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(3))
        .expect("icon atom should emit a render command");
    assert_eq!(icon.kind, UiRenderCommandKind::Image);
    assert_eq!(icon.frame, UiFrame::new(150.0, 10.0, 18.0, 18.0));
    assert_eq!(icon.text, None);
    assert_eq!(
        icon.image,
        Some(UiVisualAssetRef::Icon("layers".to_string()))
    );
    assert_eq!(icon.style.painter_family, UiPainterFamily::Generic);
    assert_eq!(icon.style.foreground_color.as_deref(), Some("#35c7d0"));
    assert_eq!(icon.opacity, 0.8);

    assert_eq!(
        commands
            .iter()
            .filter(|command| command.node_id == UiNodeId::new(2))
            .count(),
        1
    );
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.node_id == UiNodeId::new(3))
            .count(),
        1
    );
}

fn insert_control(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    component: &str,
    frame: UiFrame,
    attributes: &str,
    state_flags: UiStateFlags,
) {
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(node_id, UiNodePath::new(format!("root/{component}")))
                .with_frame(frame)
                .with_state_flags(state_flags)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}
