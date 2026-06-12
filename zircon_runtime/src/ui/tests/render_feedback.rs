use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiVisualAssetRef},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_expands_feedback_primitives() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.feedback"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 360.0, 220.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Tooltip",
        UiFrame::new(12.0, 12.0, 112.0, 52.0),
        r##"
text = "Tooltip"
label = "Shows detail"
icon = "info"
focused = true
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Toast",
        UiFrame::new(12.0, 84.0, 280.0, 32.0),
        r##"
text = "Operation completed successfully"
action = "UNDO"
hovered = true
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "Alert",
        UiFrame::new(12.0, 128.0, 280.0, 32.0),
        r##"
message = "Renderer warning"
severity = "warning"
action = "FIX"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(5),
        "AlertTitle",
        UiFrame::new(12.0, 176.0, 160.0, 24.0),
        r##"
text = "Heads up"
severity = "error"
pressed = true
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 12.0, 112.0, 52.0)
            && command.style.background_color.as_deref() == Some("#171c20")
            && command.style.border_color.as_deref() == Some("#35c7d0")
            && command.style.painter_family == UiPainterFamily::Tooltip
            && command.style.painter_state == UiPainterResolvedState::Focused
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Tooltip")
            && command.style.foreground_color.as_deref() == Some("#d0d9dd")
            && command.style.painter_family == UiPainterFamily::Tooltip
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Shows detail")
            && command.style.foreground_color.as_deref() == Some("#a8b3b8")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("info".to_string()))
            && command.style.foreground_color.as_deref() == Some("#35c7d0")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Tooltip")
            })
            .count(),
        1
    );

    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 84.0, 280.0, 32.0)
            && command.style.background_color.as_deref() == Some("#183a3f")
            && command.style.painter_family == UiPainterFamily::Toast
            && command.style.painter_state == UiPainterResolvedState::Hovered
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("check-circle".to_string()))
            && command.style.foreground_color.as_deref() == Some("#35c7d0")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Operation completed successfully")
            && command.style.foreground_color.as_deref() == Some("#cee0e2")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("UNDO")
            && command.style.foreground_color.as_deref() == Some("#35c7d0")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(3)
                    && command.text.as_deref() == Some("Operation completed successfully")
            })
            .count(),
        1
    );

    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 128.0, 280.0, 32.0)
            && command.style.background_color.as_deref() == Some("#453214")
            && command.style.border_color.as_deref() == Some("#845e23")
            && command.style.painter_family == UiPainterFamily::Alert
            && command.style.painter_state == UiPainterResolvedState::Normal
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("alert-triangle".to_string()))
            && command.style.foreground_color.as_deref() == Some("#e0a33a")
            && command.style.painter_family == UiPainterFamily::Alert
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Renderer warning")
            && command.style.foreground_color.as_deref() == Some("#e0a33a")
            && command.style.painter_family == UiPainterFamily::Alert
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("FIX")
            && command.frame == UiFrame::new(238.0, 136.8, 44.0, 14.400001)
            && command.style.foreground_color.as_deref() == Some("#e0a33a")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(4)
                    && command.text.as_deref() == Some("Renderer warning")
            })
            .count(),
        1
    );

    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(5)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Heads up")
            && command.frame == UiFrame::new(12.0, 180.2, 160.0, 15.6)
            && command.style.foreground_color.as_deref() == Some("#ef7066")
            && command.style.painter_family == UiPainterFamily::Alert
            && command.style.painter_state == UiPainterResolvedState::Pressed
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(5) && command.text.as_deref() == Some("Heads up")
            })
            .count(),
        1
    );

    let snackbar_commands = commands_for_component(
        "Snackbar",
        UiFrame::new(0.0, 0.0, 260.0, 34.0),
        r##"
message = "Background sync paused"
action = "RESUME"
icon = "pause-circle"
focused = true
"##,
        visible_state(),
    );
    assert!(snackbar_commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::Toast
            && command.style.painter_state == UiPainterResolvedState::Focused
            && command.style.border_color.as_deref() == Some("#35c7d0")
    }));
    assert!(snackbar_commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("pause-circle".to_string()))
            && command.style.painter_family == UiPainterFamily::Toast
    }));
    assert!(snackbar_commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Background sync paused")
            && command.style.painter_family == UiPainterFamily::Toast
    }));
    assert!(snackbar_commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("RESUME")
            && command.style.painter_family == UiPainterFamily::Toast
    }));

    let snackbar_content_commands = commands_for_component(
        "SnackbarContent",
        UiFrame::new(0.0, 0.0, 260.0, 34.0),
        r##"
text = "Native handoff complete"
pressed = true
"##,
        visible_state(),
    );
    assert!(snackbar_content_commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::Toast
            && command.style.painter_state == UiPainterResolvedState::Pressed
            && command.style.background_color.as_deref() == Some("#103c4a")
    }));
    assert!(snackbar_content_commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Native handoff complete")
            && command.style.painter_family == UiPainterFamily::Toast
    }));
}

#[test]
fn render_extract_loading_feedback_controls_use_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.feedback.loading"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 360.0, 160.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Tooltip",
        UiFrame::new(12.0, 12.0, 112.0, 52.0),
        r##"
text = "Tooltip"
label = "Shows detail"
icon = "info"
loading = true
pressed = true
focused = true
background_color = "#171c20"
border_color = "#252d32"
foreground_color = "#d0d9dd"
label_color = "#a8b3b8"
icon_color = "#259ca7"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Toast",
        UiFrame::new(12.0, 76.0, 280.0, 32.0),
        r##"
text = "Operation completed successfully"
action = "UNDO"
icon = "check-circle"
loading = true
hovered = true
pressed = true
background_color = "#153035"
border_color = "#35c7d014"
foreground_color = "#cee0e2"
label_color = "#35c7d0"
action_color = "#35c7d0"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "Alert",
        UiFrame::new(12.0, 120.0, 280.0, 32.0),
        r##"
message = "Renderer warning"
severity = "warning"
action = "FIX"
loading = true
hovered = true
pressed = true
background_color = "#453214"
border_color = "#845e23"
foreground_color = "#e0a33a"
icon_color = "#e0a33a"
action_color = "#e0a33a"
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::Tooltip
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Tooltip")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Shows detail")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Image
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::Toast
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Image
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Operation completed successfully")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("UNDO")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::Alert
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Image
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Renderer warning")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("FIX")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
}

fn commands_for_component(
    component: &str,
    frame: UiFrame,
    attributes: &str,
    state_flags: UiStateFlags,
) -> Vec<UiRenderCommand> {
    let mut surface = UiSurface::new(UiTreeId::new(format!(
        "runtime.ui.render.feedback.{component}"
    )));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(frame)
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        component,
        frame,
        attributes,
        state_flags,
    );
    surface.rebuild();
    surface.render_extract.list.commands
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
