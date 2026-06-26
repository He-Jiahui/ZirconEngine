use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::UiRenderCommandKind,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_expands_selection_control_indicators() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.selection_controls"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 260.0, 140.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Checkbox",
        UiFrame::new(8.0, 8.0, 120.0, 28.0),
        r##"
text = "Checkbox"
checked = true
layout_spacing = 9.0
layout_icon_size = 16.0
background_color = "#10161a"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Radio",
        UiFrame::new(8.0, 44.0, 120.0, 28.0),
        r##"
text = "Radio option"
checked = true
dot_size = 7.0
dot_color = "#43d8e2"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "Toggle",
        UiFrame::new(8.0, 80.0, 96.0, 28.0),
        r##"
text = "Switch"
checked = true
track_width = 34.0
track_height = 18.0
thumb_size = 14.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(18.0, 14.0, 16.0, 16.0)
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.as_deref() == Some("#2aa6b8")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(26.0, 18.0, 3.0, 8.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Checkbox")
            && command.frame.x >= 43.0
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Checkbox")
            })
            .count(),
        1
    );

    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(22.5, 54.5, 7.0, 7.0)
            && command.style.background_color.as_deref() == Some("#2aa6b8")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Radio option")
    }));

    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(62.0, 85.0, 34.0, 18.0)
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.as_deref() == Some("#2aa6b8")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(80.0, 87.0, 14.0, 14.0)
            && command.style.background_color.as_deref() == Some("#e8ecee")
    }));
}

#[test]
fn render_extract_uses_shared_selector_for_pressed_checked_selection_border() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.selection_controls.selector",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 64.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Checkbox",
        UiFrame::new(8.0, 8.0, 120.0, 28.0),
        r##"
text = "Pressed"
checked = true
"##,
        UiStateFlags {
            pressed: true,
            ..visible_state()
        },
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(18.0, 14.0, 16.0, 16.0)
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.as_deref() == Some("#35c7d0")
    }));
}

#[test]
fn render_extract_loading_selection_controls_use_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.selection_controls.loading",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 132.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Checkbox",
        UiFrame::new(8.0, 8.0, 150.0, 28.0),
        r##"
text = "Checkbox"
checked = true
selected = true
pressed = true
hovered = true
loading = true
background_color = "#10161a"
border_color = "#35c7d0"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Radio",
        UiFrame::new(8.0, 44.0, 150.0, 28.0),
        r##"
text = "Radio"
checked = true
pressed = true
hovered = true
loading = true
dot_color = "#43d8e2"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "Toggle",
        UiFrame::new(8.0, 80.0, 110.0, 28.0),
        r##"
text = "Toggle"
checked = true
pressed = true
hovered = true
loading = true
background_color = "#209fa8"
border_color = "#35c7d0"
foreground_color = "#ffffff"
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(18.0, 14.0, 16.0, 16.0)
            && command.style.painter_family == UiPainterFamily::Checkbox
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(26.0, 18.0, 3.0, 8.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(18.0, 50.0, 16.0, 16.0)
            && command.style.painter_family == UiPainterFamily::Radio
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(22.5, 54.5, 7.0, 7.0)
            && command.style.painter_family == UiPainterFamily::Radio
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(76.0, 85.0, 34.0, 18.0)
            && command.style.painter_family == UiPainterFamily::Toggle
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(94.0, 87.0, 14.0, 14.0)
            && command.style.painter_family == UiPainterFamily::Toggle
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#59656c")
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
