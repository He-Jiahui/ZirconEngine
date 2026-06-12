use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommandKind, UiVisualAssetRef},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_expands_button_primitives() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.buttons"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 120.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Button",
        UiFrame::new(12.0, 16.0, 132.0, 30.0),
        r##"
text = "Compile"
icon = "play"
button_color = "primary"
layout_padding_left = 12.0
layout_padding_right = 12.0
layout_spacing = 7.0
layout_icon_size = 16.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 16.0, 132.0, 30.0)
            && command.style.background_color.as_deref() == Some("#32b8c5")
            && command.style.border_color.as_deref() == Some("#249aa6")
            && command.style.painter_family == UiPainterFamily::Button
            && command.style.painter_state == UiPainterResolvedState::Normal
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("play".to_string()))
            && command.frame == UiFrame::new(24.0, 23.0, 16.0, 16.0)
            && command.style.foreground_color.as_deref() == Some("#08181b")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Compile")
            && command.frame == UiFrame::new(47.0, 24.4, 85.0, 13.200001)
            && command.style.foreground_color.as_deref() == Some("#08181b")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Compile")
            })
            .count(),
        1
    );
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2)
                    && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("play".to_string()))
            })
            .count(),
        1
    );
}

#[test]
fn render_extract_expands_icon_button_state_surface() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.icon_buttons"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 80.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "IconButton",
        UiFrame::new(16.0, 20.0, 40.0, 40.0),
        r##"
icon = "transform"
label = "Move"
selected = true
layout_icon_size = 18.0
corner_radius = 6.0
"##,
        UiStateFlags {
            hoverable: true,
            clickable: true,
            focusable: true,
            ..visible_state()
        },
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(16.0, 20.0, 40.0, 40.0)
            && command.style.background_color.as_deref() == Some("#14373c")
            && command.style.border_color.as_deref() == Some("#35c7d0")
            && command.style.corner_radius == 6.0
            && command.style.painter_family == UiPainterFamily::IconButton
            && command.style.painter_state == UiPainterResolvedState::Selected
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("transform".to_string()))
            && command.frame == UiFrame::new(27.0, 31.0, 18.0, 18.0)
            && command.style.foreground_color.as_deref() == Some("#35c7d0")
            && command.style.painter_state == UiPainterResolvedState::Selected
    }));
    assert!(
        commands
            .iter()
            .all(|command| command.node_id != UiNodeId::new(2) || command.text.is_none()),
        "icon-only buttons should not render accessibility labels as visible text"
    );
}

#[test]
fn render_extract_loading_button_and_icon_button_use_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.buttons.loading"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 260.0, 120.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Button",
        UiFrame::new(12.0, 16.0, 132.0, 30.0),
        r##"
text = "Compile"
icon = "play"
button_color = "primary"
loading = true
hovered = true
focused = true
pressed = true
background_color = "#32b8c5"
border_color = "#249aa6"
foreground_color = "#08181b"
layout_padding_left = 12.0
layout_padding_right = 12.0
layout_spacing = 7.0
layout_icon_size = 16.0
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "IconButton",
        UiFrame::new(160.0, 16.0, 40.0, 40.0),
        r##"
icon = "trash"
selected = true
checked = true
loading = true
hovered = true
focused = true
pressed = true
background_color = "#14373c"
border_color = "#35c7d0"
icon_color = "#ef7066"
selected_icon_color = "#35c7d0"
layout_icon_size = 18.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::Button
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("play".to_string()))
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Compile")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::IconButton
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("trash".to_string()))
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
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
