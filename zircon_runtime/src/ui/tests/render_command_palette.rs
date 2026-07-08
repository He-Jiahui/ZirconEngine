use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_command_palette_draws_search_panel_and_filtered_command_rows() {
    let commands = commands_for_command_palette(
        UiFrame::new(40.0, 32.0, 320.0, 160.0),
        r##"
open = true
popup_open = true
text = "Command Palette"
query = "build"
placeholder = "Search commands"
command_source = "workbench"
commands = [
  { id = "open_scene", label = "Open Scene", source = "workbench", shortcut = "Ctrl+O" },
  { id = "build_project", label = "Build Project", source = "workbench", shortcut = "Ctrl+B" },
  { id = "build_assets", label = "Build Assets", source = "workbench", shortcut = "Ctrl+Shift+B", disabled = true },
  { id = "reload_runtime", label = "Reload Runtime", source = "runtime", shortcut = "Ctrl+R" },
]
filtered_commands = ["build_project", "build_assets"]
disabled_commands = ["build_assets"]
selected_command_id = "build_project"
focused_index = 0
"##,
    );

    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(40.0, 32.0, 320.0, 160.0)
            && command.style.background_color.as_deref() == Some("#151b1f")
            && command.style.border_color.as_deref() == Some("#303840")
            && command.style.corner_radius == 6.0
            && command.style.painter_family == UiPainterFamily::Dropdown
            && command.style.painter_state == UiPainterResolvedState::Open
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(52.0, 42.0, 296.0, 30.0)
            && command.style.background_color.as_deref() == Some("#10161a")
            && command.style.border_color.as_deref() == Some("#35c7d0")
            && command.style.painter_family == UiPainterFamily::TextField
            && command.style.painter_state == UiPainterResolvedState::Focused
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("build")
            && command.frame == UiFrame::new(62.0, 49.0, 276.0, 14.4)
            && command.style.foreground_color.as_deref() == Some("#e8ecee")
            && command.style.painter_family == UiPainterFamily::TextField
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Build Project")
            && command.frame == UiFrame::new(57.0, 85.0, 286.0, 16.0)
            && command.style.foreground_color.as_deref() == Some("#35c7d0")
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Focused
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Build Assets")
            && command.frame == UiFrame::new(57.0, 111.0, 286.0, 16.0)
            && command.style.foreground_color.as_deref() == Some("#59656c")
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Disabled
    }));
    assert!(!commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Open Scene")));
    assert!(!commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Reload Runtime")));
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.text.as_deref() == Some("Command Palette"))
            .count(),
        0,
        "CommandPalette should suppress generic owner text rendering"
    );
}

#[test]
fn render_extract_command_palette_focused_row_keeps_neutral_popup_surface_until_hovered() {
    let commands = commands_for_command_palette(
        UiFrame::new(40.0, 32.0, 320.0, 160.0),
        r##"
open = true
popup_open = true
query = "build"
placeholder = "Search commands"
commands = [
  { id = "build_project", label = "Build Project", source = "workbench", shortcut = "Ctrl+B" },
  { id = "build_assets", label = "Build Assets", source = "workbench", shortcut = "Ctrl+Shift+B" },
]
filtered_commands = ["build_project", "build_assets"]
focused_index = 0
"##,
    );

    let focused_surface = commands
        .iter()
        .find(|command| {
            command.kind == UiRenderCommandKind::Quad
                && command.frame == UiFrame::new(48.0, 80.0, 304.0, 26.0)
                && command.style.painter_family == UiPainterFamily::PopupRow
        })
        .expect("focused-only command palette row should keep a neutral popup row surface");
    assert_eq!(
        focused_surface.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_surface.style.background_color.as_deref(),
        Some("#151b1f")
    );
    assert_eq!(
        focused_surface.style.border_color.as_deref(),
        Some("#303840")
    );
    assert_eq!(focused_surface.style.border_width, 1.0);
    assert!(!commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(48.0, 80.0, 304.0, 26.0)
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.background_color.as_deref() == Some("#1a2429")
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Build Project")
            && command.frame == UiFrame::new(57.0, 85.0, 286.0, 16.0)
            && command.style.foreground_color.as_deref() == Some("#c5d0d5")
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Focused
    }));
}

fn commands_for_command_palette(frame: UiFrame, attributes: &str) -> Vec<UiRenderCommand> {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.command_palette"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 640.0, 360.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/CommandPalette"))
                .with_frame(frame)
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "CommandPalette".to_string(),
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
