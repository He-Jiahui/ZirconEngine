use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_dialog_uses_modal_panel_commands_without_owner_text_duplication() {
    let commands = commands_for_component(
        "Dialog",
        UiFrame::new(110.0, 70.0, 420.0, 200.0),
        r##"
open = true
popup_open = true
text = "Scene Settings"
title = "Scene Settings"
message = "Review scene-level settings before applying changes."
"##,
        visible_state(),
    );

    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(110.0, 70.0, 420.0, 200.0)
            && command.style.background_color.as_deref() == Some("#171c20")
            && command.style.border_color.as_deref() == Some("#35c7d0")
            && command.style.corner_radius == 6.0
            && command.style.painter_family == UiPainterFamily::Alert
            && command.style.painter_state == UiPainterResolvedState::Open
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Scene Settings")
            && command.frame == UiFrame::new(130.0, 88.0, 380.0, 18.0)
            && command.style.foreground_color.as_deref() == Some("#e8ecee")
            && command.style.painter_family == UiPainterFamily::Alert
            && command.style.painter_state == UiPainterResolvedState::Open
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command
                .text
                .as_deref()
                .is_some_and(|text| text.starts_with("Review scene-level settings"))
            && command.frame == UiFrame::new(130.0, 118.0, 380.0, 16.0)
            && command.style.foreground_color.as_deref() == Some("#a4aeb4")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.text.as_deref() == Some("Scene Settings"))
            .count(),
        1,
        "Dialog render extraction should suppress the generic owner text command"
    );
}

#[test]
fn render_extract_confirm_dialog_projects_severity_and_disabled_confirm_action() {
    let commands = commands_for_component(
        "ConfirmDialog",
        UiFrame::new(80.0, 48.0, 460.0, 210.0),
        r##"
open = true
popup_open = true
title = "Delete selected node?"
message = "This removes the node from the scene hierarchy."
confirm_text = "Delete"
cancel_text = "Cancel"
severity = "error"
destructive = true
confirm_enabled = false
"##,
        visible_state(),
    );

    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(80.0, 48.0, 460.0, 210.0)
            && command.style.background_color.as_deref() == Some("#171c20")
            && command.style.border_color.as_deref() == Some("#853d3a")
            && command.style.painter_family == UiPainterFamily::Alert
            && command.style.painter_state == UiPainterResolvedState::Open
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(80.0, 48.0, 4.0, 210.0)
            && command.style.background_color.as_deref() == Some("#ef7066")
            && command.style.painter_family == UiPainterFamily::Alert
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Delete selected node?")
            && command.style.foreground_color.as_deref() == Some("#ef7066")
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Cancel")
            && command.style.foreground_color.as_deref() == Some("#a4aeb4")
            && command.style.painter_state == UiPainterResolvedState::Open
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Delete")
            && command.style.foreground_color.as_deref() == Some("#59656c")
            && command.style.painter_state == UiPainterResolvedState::Disabled
    }));
}

fn commands_for_component(
    component: &str,
    frame: UiFrame,
    attributes: &str,
    state_flags: UiStateFlags,
) -> Vec<UiRenderCommand> {
    let mut surface = UiSurface::new(UiTreeId::new(format!(
        "runtime.ui.render.dialog.{component}"
    )));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 640.0, 360.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(
                UiNodeId::new(2),
                UiNodePath::new(format!("root/{component}")),
            )
            .with_frame(frame)
            .with_state_flags(state_flags)
            .with_template_metadata(UiTemplateNodeMetadata {
                component: component.to_string(),
                attributes: toml::from_str(attributes).unwrap(),
                ..UiTemplateNodeMetadata::default()
            }),
        )
        .unwrap();
    surface.rebuild();
    surface.render_extract.list.commands
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}
