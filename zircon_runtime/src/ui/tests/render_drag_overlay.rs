use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiVisualAssetRef},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_drag_overlay_draws_preview_chip_and_drop_indicator() {
    let commands = commands_for_drag_overlay(
        UiFrame::new(0.0, 0.0, 360.0, 220.0),
        r##"
open = true
dragging = true
text = "Fallback drag text"
payload_kind = "asset"
payload_label = "StoneWall.mesh"
payload_reference = "assets/stone_wall.mesh"
cursor_x = 72.0
cursor_y = 48.0
offset_x = 16.0
offset_y = 18.0
preview_width = 184.0
preview_height = 36.0
drop_hovered = true
drop_allowed = true
drop_target_x = 24.0
drop_target_y = 148.0
drop_target_width = 280.0
drop_target_height = 30.0
drop_indicator_edge = "bottom"
"##,
    );

    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(88.0, 66.0, 184.0, 36.0)
            && command.style.background_color.as_deref() == Some("#153035")
            && command.style.border_color.as_deref() == Some("#35c7d0")
            && command.style.corner_radius == 6.0
            && command.style.painter_family == UiPainterFamily::Chrome
            && command.style.painter_state == UiPainterResolvedState::Dragging
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Image
            && command.frame == UiFrame::new(100.0, 75.0, 18.0, 18.0)
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("package".to_string()))
            && command.style.foreground_color.as_deref() == Some("#35c7d0")
            && command.style.painter_family == UiPainterFamily::Chrome
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("StoneWall.mesh")
            && command.frame == UiFrame::new(126.0, 76.8, 134.0, 14.4)
            && command.style.foreground_color.as_deref() == Some("#cee0e2")
            && command.style.painter_family == UiPainterFamily::Chrome
            && command.style.painter_state == UiPainterResolvedState::Dragging
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(24.0, 176.0, 280.0, 2.0)
            && command.style.background_color.as_deref() == Some("#35c7d0")
            && command.style.painter_family == UiPainterFamily::Chrome
            && command.style.painter_state == UiPainterResolvedState::DropHovered
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.text.as_deref() == Some("Fallback drag text"))
            .count(),
        0,
        "DragOverlay should suppress generic owner text rendering"
    );
}

#[test]
fn render_extract_closed_drag_overlay_suppresses_owner_fallback() {
    let commands = commands_for_drag_overlay(
        UiFrame::new(0.0, 0.0, 200.0, 80.0),
        r##"
open = false
dragging = false
text = "Should not render"
background_color = "#ff00ff"
"##,
    );

    assert!(
        commands
            .iter()
            .all(|command| command.kind == UiRenderCommandKind::Group),
        "closed DragOverlay should be paint-silent"
    );
}

fn commands_for_drag_overlay(frame: UiFrame, attributes: &str) -> Vec<UiRenderCommand> {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.drag_overlay"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 640.0, 360.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/DragOverlay"))
                .with_frame(frame)
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "DragOverlay".to_string(),
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
