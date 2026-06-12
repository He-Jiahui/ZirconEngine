use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiVisualAssetRef},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_expands_workbench_chrome_surfaces() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.chrome"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 620.0, 260.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "WorkbenchShell",
        UiFrame::new(0.0, 0.0, 620.0, 260.0),
        r##"
label = "Editor"
icon = "zircon-logo"
background_color = "#111820"
border_color = "#25313a"
border_width = 1.0
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "ActivityRail",
        UiFrame::new(0.0, 32.0, 44.0, 200.0),
        r##"
label = "Tools"
icon = "panel-left"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "TopToolbar",
        UiFrame::new(44.0, 0.0, 500.0, 32.0),
        r##"
text = "Main Toolbar"
separator_edge = "bottom"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(5),
        "ViewportPanel",
        UiFrame::new(54.0, 42.0, 360.0, 180.0),
        r##"
title = "Scene"
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let shell_surface = chrome_surface(commands, UiNodeId::new(2));
    assert_eq!(shell_surface.kind, UiRenderCommandKind::Quad);
    assert_eq!(shell_surface.style.painter_family, UiPainterFamily::Chrome);
    assert_eq!(
        shell_surface.style.painter_state,
        UiPainterResolvedState::Normal
    );
    assert_eq!(
        shell_surface.style.background_color.as_deref(),
        Some("#111820")
    );
    assert_eq!(shell_surface.style.border_color.as_deref(), Some("#25313a"));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.style.painter_family == UiPainterFamily::Chrome
            && command.text.as_deref() == Some("Editor")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Image
            && command.style.painter_family == UiPainterFamily::Chrome
            && command.image == Some(UiVisualAssetRef::Icon("zircon-logo".to_string()))
    }));

    let rail_surface = chrome_surface(commands, UiNodeId::new(3));
    assert_eq!(
        rail_surface.style.background_color.as_deref(),
        Some("#1b2226")
    );
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::Chrome
            && command.frame == UiFrame::new(43.0, 32.0, 1.0, 200.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::Chrome
            && command.frame == UiFrame::new(44.0, 31.0, 500.0, 1.0)
    }));

    let viewport_surface = chrome_surface(commands, UiNodeId::new(5));
    assert_eq!(
        viewport_surface.style.background_color.as_deref(),
        Some("#0b1115")
    );
    assert_eq!(viewport_surface.style.border_width, 0.0);
}

#[test]
fn render_extract_chrome_uses_shared_unavailable_and_active_state_priority() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.chrome.state"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 400.0, 130.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "InspectorPanel",
        UiFrame::new(8.0, 8.0, 180.0, 80.0),
        r##"
title = "Inspector"
selected_background_color = "#184c54"
focus_border_color = "#35c7d0"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "StatusBar",
        UiFrame::new(0.0, 100.0, 400.0, 24.0),
        r##"
text = "Saving"
loading = true
background_color = "#008800"
foreground_color = "#ffffff"
"##,
        visible_state(),
    );

    assert!(surface.component_states.set_focused(UiNodeId::new(2), true));
    surface
        .mark_component_state_render_dirty(UiNodeId::new(2))
        .unwrap();
    surface.rebuild();

    let panel_surface = chrome_surface(&surface.render_extract.list.commands, UiNodeId::new(2));
    assert_eq!(
        panel_surface.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        panel_surface.style.background_color.as_deref(),
        Some("#184c54")
    );
    assert_eq!(panel_surface.style.border_color.as_deref(), Some("#35c7d0"));

    let status_surface = chrome_surface(&surface.render_extract.list.commands, UiNodeId::new(3));
    assert_eq!(
        status_surface.style.painter_state,
        UiPainterResolvedState::Loading
    );
    assert_eq!(
        status_surface.style.background_color.as_deref(),
        Some("#20262a")
    );
    assert!(surface.render_extract.list.commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.style.painter_family == UiPainterFamily::Chrome
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

fn chrome_surface(commands: &[UiRenderCommand], node_id: UiNodeId) -> &UiRenderCommand {
    commands
        .iter()
        .find(|command| {
            command.node_id == node_id
                && command.kind == UiRenderCommandKind::Quad
                && command.style.painter_family == UiPainterFamily::Chrome
                && command.frame.width > 1.0
                && command.frame.height > 1.0
        })
        .expect("chrome surface command")
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}
