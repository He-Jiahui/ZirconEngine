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
fn notification_rendering_moves_row_text_and_avoids_lowercase_allocations() {
    let source = include_str!("../surface/render/notification_center.rs");

    assert!(!source.contains("row.title.clone()"));
    assert!(!source.contains("row.message.clone()"));
    assert!(!source.contains("to_ascii_lowercase"));
    assert!(source.contains("EditorDesignTokens"));
    assert!(!source.contains("const PANEL_SURFACE"));
    assert!(!source.contains("const HEADER_TEXT"));
}

#[test]
fn render_extract_notification_center_draws_panel_header_and_notifications() {
    let commands = commands_for_notification_center(
        UiFrame::new(32.0, 24.0, 300.0, 160.0),
        r##"
open = true
popup_open = true
text = "Owner fallback"
title = "Notifications"
unread_count = 2
focused_index = 1
selected_notification_id = "build"
visible_limit = 3
notifications = [
  { id = "build", title = "Build failed", message = "Shader compile error", severity = "error", unread = true },
  { id = "asset", title = "Asset imported", message = "StoneWall.mesh ready", severity = "success" },
]
"##,
    );

    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(32.0, 24.0, 300.0, 160.0)
            && command.style.background_color.as_deref() == Some("#141618")
            && command.style.border_color.as_deref() == Some("#323a41")
            && command.style.corner_radius == 8.0
            && command.style.painter_family == UiPainterFamily::Toast
            && command.style.painter_state == UiPainterResolvedState::Open
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Notifications (2)")
            && command.frame == UiFrame::new(44.0, 34.0, 276.0, 16.0)
            && command.style.foreground_color.as_deref() == Some("#e8ecee")
            && command.style.font_size == EditorTypographyTokens::WORKBENCH_BODY_SIZE
            && command.style.painter_family == UiPainterFamily::Toast
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(40.0, 60.0, 284.0, 48.0)
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
            && command.style.painter_state == UiPainterResolvedState::Selected
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(50.0, 68.0, 3.0, 32.0)
            && command.style.background_color.as_deref() == Some("#eb605c")
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Build failed")
            && command.frame
                == UiFrame::new(
                    62.0,
                    67.0,
                    250.0,
                    EditorTypographyTokens::WORKBENCH_OVERLAY_SIZE
                        * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
                )
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Shader compile error")
            && command.frame
                == UiFrame::new(
                    62.0,
                    85.0,
                    250.0,
                    EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
                        * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
                )
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(40.0, 114.0, 284.0, 48.0)
            && command.style.background_color.as_deref() == Some("#171a1d")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
            && command.style.painter_state == UiPainterResolvedState::Focused
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(50.0, 122.0, 3.0, 32.0)
            && command.style.background_color.as_deref() == Some("#55be78")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.text.as_deref() == Some("Owner fallback"))
            .count(),
        0,
        "NotificationCenter should suppress generic owner text rendering"
    );
}

#[test]
fn render_extract_notification_center_keeps_focused_rows_neutral_until_selected() {
    let focused_only = commands_for_notification_center(
        UiFrame::new(20.0, 12.0, 260.0, 120.0),
        r##"
open = true
notifications = [
  { id = "compile", title = "Compile queued", message = "Waiting", severity = "info" }
]
focused_index = 0
"##,
    );

    let focused_row =
        notification_row_surface(&focused_only, UiFrame::new(28.0, 48.0, 244.0, 48.0));
    assert_eq!(
        focused_row.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_row.style.background_color.as_deref(),
        Some("#171a1d")
    );
    assert_eq!(focused_row.style.border_color.as_deref(), Some("#3cc7d6"));
    assert!(!focused_only.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(28.0, 48.0, 244.0, 48.0)
            && command.style.background_color.as_deref() == Some("#183a3f")
    }));

    let selected_and_focused = commands_for_notification_center(
        UiFrame::new(20.0, 12.0, 260.0, 120.0),
        r##"
open = true
selected_notification_id = "compile"
focused_index = 0
notifications = [
  { id = "compile", title = "Compile queued", message = "Waiting", severity = "info", unread = true }
]
"##,
    );

    let selected_row =
        notification_row_surface(&selected_and_focused, UiFrame::new(28.0, 48.0, 244.0, 48.0));
    assert_eq!(
        selected_row.style.painter_state,
        UiPainterResolvedState::Selected
    );
    assert_eq!(
        selected_row.style.background_color.as_deref(),
        Some("#153035")
    );
    assert_eq!(selected_row.style.border_color.as_deref(), Some("#3cc7d6"));
}

#[test]
fn render_extract_notification_center_consumes_resolved_template_visual_tokens() {
    let commands = commands_for_notification_center(
        UiFrame::new(20.0, 12.0, 260.0, 120.0),
        r##"
open = true
panel_surface_color = "#010203"
panel_border_color = "#040506"
header_text_color = "#070809"
accent_color = "#a0b1c2"
panel_radius = 3.0
header_font_size = 15.0
typography_line_height_ratio = 1.2
selected_notification_id = "compile"
notifications = [
  { id = "compile", title = "Compile queued", message = "Waiting", severity = "info" }
]
"##,
    );

    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(20.0, 12.0, 260.0, 120.0)
            && command.style.background_color.as_deref() == Some("#010203")
            && command.style.border_color.as_deref() == Some("#040506")
            && command.style.corner_radius == 3.0
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Notifications")
            && command.style.foreground_color.as_deref() == Some("#070809")
            && command.style.font_size == 15.0
            && command.style.line_height == 18.0
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(28.0, 48.0, 244.0, 48.0)
            && command.style.border_color.as_deref() == Some("#a0b1c2")
            && command.style.painter_state == UiPainterResolvedState::Selected
    }));
}

#[test]
fn render_extract_closed_notification_center_suppresses_owner_fallback() {
    let commands = commands_for_notification_center(
        UiFrame::new(0.0, 0.0, 220.0, 80.0),
        r##"
open = false
popup_open = false
text = "Should not render"
background_color = "#ff00ff"
"##,
    );

    assert!(
        commands
            .iter()
            .all(|command| command.kind == UiRenderCommandKind::Group),
        "closed NotificationCenter should be paint-silent"
    );
}

fn commands_for_notification_center(frame: UiFrame, attributes: &str) -> Vec<UiRenderCommand> {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.notification_center"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 640.0, 360.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/NotificationCenter"))
                .with_frame(frame)
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "NotificationCenter".to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface.render_extract.list.commands
}

fn notification_row_surface(commands: &[UiRenderCommand], frame: UiFrame) -> &UiRenderCommand {
    commands
        .iter()
        .find(|command| {
            command.kind == UiRenderCommandKind::Quad
                && command.frame == frame
                && command.style.painter_family == UiPainterFamily::Toast
        })
        .expect("notification row surface should render")
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}
