use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::UiRenderCommandKind,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_expands_open_context_action_menu_items() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.popup_menu"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 140.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/menu"))
                .with_frame(UiFrame::new(8.0, 8.0, 160.0, 96.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ContextActionMenu".to_string(),
                    attributes: toml::from_str(
                        r##"
value = "Play In Editor"
popup_open = true
menu_items = ["Play In Editor|checked,icon=play", "Simulate", "---", "Network Preview|submenu"]
background_color = "#151b1f"
border_color = "#303840"
border_width = 1.0
corner_radius = 5.0
"##,
                    )
                    .unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let owner = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Play In Editor")
        })
        .expect("open menu owner should keep its visible value command");
    assert_eq!(owner.kind, UiRenderCommandKind::Quad);

    let expanded_item_texts = commands
        .iter()
        .filter(|command| command.node_id == UiNodeId::new(2))
        .filter(|command| matches!(command.kind, UiRenderCommandKind::Text))
        .filter_map(|command| command.text.as_deref())
        .collect::<Vec<_>>();
    assert!(expanded_item_texts.contains(&"Simulate"));
    assert!(expanded_item_texts.contains(&"Network Preview"));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.z_index > owner.z_index
            && command.frame == UiFrame::new(8.0, 8.0, 160.0, 96.0)
    }));
}

#[test]
fn render_extract_loading_context_menu_item_uses_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.popup_menu.loading"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 140.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/menu"))
                .with_frame(UiFrame::new(8.0, 8.0, 160.0, 72.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ContextActionMenu".to_string(),
                    attributes: toml::from_str(
                        r##"
popup_open = true
menu_items = ["Delete|checked,hovered,pressed,danger,loading", "Simulate"]
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
            && command.text.as_deref() == Some("Delete")
            && command.frame == UiFrame::new(17.0, 13.0, 142.0, 26.0)
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.frame == UiFrame::new(8.0, 8.0, 160.0, 36.0)
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.frame == UiFrame::new(8.0, 12.0, 3.0, 28.0)
    }));
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}
