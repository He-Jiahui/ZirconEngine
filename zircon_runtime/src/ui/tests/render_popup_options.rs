use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::UiRenderCommandKind,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_expands_open_dropdown_options() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.popup_options"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 180.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/dropdown"))
                .with_frame(UiFrame::new(12.0, 16.0, 148.0, 28.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Dropdown".to_string(),
                    attributes: toml::from_str(
                        r##"
value = "surface"
value_text = "Surface"
popup_open = true
options = ["surface|label=Surface", "post_process|label=Post Process", "volume|label=Volume"]
hovered_options = ["post_process"]
disabled_options = ["volume"]
background_color = "#10161a"
border_color = "#323f47"
border_width = 1.0
corner_radius = 4.0
"##,
                    )
                    .unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .expect("dropdown node should exist")
        .layout_cache
        .clip_frame = Some(UiFrame::new(12.0, 16.0, 148.0, 28.0));

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let owner = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("surface")
        })
        .expect("open dropdown owner should keep its visible value command");
    assert_eq!(owner.kind, UiRenderCommandKind::Quad);

    let option_texts = commands
        .iter()
        .filter(|command| command.node_id == UiNodeId::new(2))
        .filter(|command| matches!(command.kind, UiRenderCommandKind::Text))
        .filter_map(|command| command.text.as_deref())
        .collect::<Vec<_>>();
    assert!(option_texts.contains(&"Surface"));
    assert!(option_texts.contains(&"Post Process"));
    assert!(option_texts.contains(&"Volume"));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.z_index > owner.z_index
            && command.frame == UiFrame::new(12.0, 48.0, 148.0, 84.0)
            && command.clip_frame.is_none()
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.z_index > owner.z_index
            && command.frame == UiFrame::new(12.0, 48.0, 148.0, 28.0)
    }));
}

#[test]
fn render_extract_loading_dropdown_option_uses_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.popup_options.loading"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 180.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/dropdown"))
                .with_frame(UiFrame::new(12.0, 16.0, 148.0, 28.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Dropdown".to_string(),
                    attributes: toml::from_str(
                        r##"
value = "surface"
value_text = "Surface"
popup_open = true
options = ["surface|label=Surface,loading,hovered,pressed,special", "post_process|label=Post Process"]
"##,
                    )
                    .unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Surface")
            && command.frame == UiFrame::new(21.0, 53.0, 130.0, 18.0)
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.frame == UiFrame::new(12.0, 48.0, 148.0, 28.0)
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.frame == UiFrame::new(12.0, 52.0, 3.0, 20.0)
    }));
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}
