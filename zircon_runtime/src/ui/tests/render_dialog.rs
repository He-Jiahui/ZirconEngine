use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    design_tokens::EditorTypographyTokens,
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn dialog_severity_parsing_does_not_allocate_lowercase_text() {
    let source = include_str!("../surface/render/dialog.rs");

    assert!(!source.contains("to_ascii_lowercase"));
    assert!(source.contains("EditorDesignTokens"));
    assert!(!source.contains("const DIALOG_SURFACE"));
    assert!(!source.contains("const DIALOG_TITLE_FONT_SIZE"));
}

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
            && command.style.background_color.as_deref() == Some("#141618")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
            && command.style.corner_radius == 8.0
            && command.style.painter_family == UiPainterFamily::Alert
            && command.style.painter_state == UiPainterResolvedState::Open
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Scene Settings")
            && command.frame
                == UiFrame::new(
                    130.0,
                    88.0,
                    380.0,
                    EditorTypographyTokens::WORKBENCH_TITLE_SIZE
                        * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
                )
            && command.style.foreground_color.as_deref() == Some("#e8ecee")
            && command.style.font_size == EditorTypographyTokens::WORKBENCH_TITLE_SIZE
            && command.style.painter_family == UiPainterFamily::Alert
            && command.style.painter_state == UiPainterResolvedState::Open
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command
                .text
                .as_deref()
                .is_some_and(|text| text.starts_with("Review scene-level settings"))
            && command.frame
                == UiFrame::new(
                    130.0,
                    118.0,
                    380.0,
                    EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
                        * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
                )
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
            && command.style.background_color.as_deref() == Some("#141618")
            && command.style.border_color.as_deref() == Some("#4c2427")
            && command.style.painter_family == UiPainterFamily::Alert
            && command.style.painter_state == UiPainterResolvedState::Open
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(80.0, 48.0, 4.0, 210.0)
            && command.style.background_color.as_deref() == Some("#eb605c")
            && command.style.painter_family == UiPainterFamily::Alert
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Delete selected node?")
            && command.style.foreground_color.as_deref() == Some("#eb605c")
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
            && command.style.foreground_color.as_deref() == Some("#656f76")
            && command.style.painter_state == UiPainterResolvedState::Disabled
    }));
}

#[test]
fn render_extract_dialog_action_width_uses_runtime_text_measurement() {
    let narrow = dialog_action_command("iiiiiiiiiiii");
    let wide = dialog_action_command("WWWWWWWWWWWW");

    assert_eq!(narrow.text.as_deref(), Some("iiiiiiiiiiii"));
    assert_eq!(wide.text.as_deref(), Some("WWWWWWWWWWWW"));
    assert!(
        wide.frame.width > narrow.frame.width + 8.0,
        "same-character-count dialog action labels should be sized by runtime glyph measurement, narrow={:?}, wide={:?}",
        narrow.frame,
        wide.frame
    );
}

#[test]
fn repeated_dialog_action_measurement_uses_the_surface_frame_owner() {
    let surface = surface_for_component(
        "ConfirmDialog",
        UiFrame::new(80.0, 48.0, 460.0, 210.0),
        r##"
open = true
popup_open = true
confirm_text = "Same action"
cancel_text = "Same action"
"##,
        visible_state(),
    );

    assert_eq!(
        surface
            .text_measure_cache
            .frame_measure_dedup_report()
            .hit_count,
        1,
        "confirm and cancel sizing must share the extraction frame's text measurement owner"
    );
}

#[test]
fn render_extract_dialog_consumes_resolved_template_visual_tokens() {
    let commands = commands_for_component(
        "Dialog",
        UiFrame::new(110.0, 70.0, 420.0, 200.0),
        r##"
open = true
title = "Tokenized dialog"
message = "Template visual values override the central defaults."
background_color = "#010203"
focus_border_color = "#040506"
title_color = "#070809"
corner_radius = 3.0
title_font_size = 15.0
typography_line_height_ratio = 1.2
"##,
        visible_state(),
    );

    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(110.0, 70.0, 420.0, 200.0)
            && command.style.background_color.as_deref() == Some("#010203")
            && command.style.border_color.as_deref() == Some("#040506")
            && command.style.corner_radius == 3.0
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Tokenized dialog")
            && command.style.foreground_color.as_deref() == Some("#070809")
            && command.style.font_size == 15.0
            && command.style.line_height == 18.0
    }));
}

fn commands_for_component(
    component: &str,
    frame: UiFrame,
    attributes: &str,
    state_flags: UiStateFlags,
) -> Vec<UiRenderCommand> {
    surface_for_component(component, frame, attributes, state_flags)
        .render_extract
        .list
        .commands
}

fn surface_for_component(
    component: &str,
    frame: UiFrame,
    attributes: &str,
    state_flags: UiStateFlags,
) -> UiSurface {
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
    surface
}

fn dialog_action_command(action: &str) -> UiRenderCommand {
    commands_for_component(
        "Dialog",
        UiFrame::new(100.0, 64.0, 460.0, 210.0),
        &format!(
            r##"
open = true
popup_open = true
title = "Measured action"
message = "Action frame width follows runtime text measurement."
action = "{action}"
"##
        ),
        visible_state(),
    )
    .into_iter()
    .find(|command| {
        command.kind == UiRenderCommandKind::Text && command.text.as_deref() == Some(action)
    })
    .expect("dialog action text command")
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}
