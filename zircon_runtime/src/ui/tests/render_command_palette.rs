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
fn command_palette_filtering_compares_borrowed_ascii_text_and_moves_labels() {
    let source = include_str!("../surface/render/command_palette.rs");

    assert!(!source.contains("to_ascii_lowercase"));
    assert!(!source.contains("command.label.clone()"));
    assert!(source.contains("EditorDesignTokens"));
    assert!(source.contains("EditorTypographyTokens"));
    assert!(!source.contains("const PANEL_SURFACE"));
    assert!(!source.contains("const TEXT_FONT_SIZE"));
    assert!(source.contains("style_overrides"));
    assert!(source.contains("value_as_f32"));
}

#[test]
fn anchored_command_palette_consumes_runtime_popup_geometry() {
    let commands = commands_for_command_palette(
        UiFrame::new(40.0, 32.0, 320.0, 160.0),
        r##"
open = true
popup_open = true
placement = "top"
popup_anchor_x = 20.0
popup_anchor_y = 64.0
popup_anchor_width = 600.0
popup_anchor_height = 0.0
anchor_origin_vertical = "top"
anchor_origin_horizontal = "center"
transform_origin_vertical = "top"
transform_origin_horizontal = "center"
popup_offset_y = 24.0
"##,
    );

    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::Dropdown
            && command.frame == UiFrame::new(160.0, 88.0, 320.0, 160.0)
            && command.clip_frame == Some(UiFrame::new(0.0, 0.0, 640.0, 360.0))
    }));
}

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
            && command.style.background_color.as_deref() == Some("#141618")
            && command.style.border_color.as_deref() == Some("#323a41")
            && command.style.corner_radius == 8.0
            && command.style.painter_family == UiPainterFamily::Dropdown
            && command.style.painter_state == UiPainterResolvedState::Open
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(52.0, 40.0, 296.0, 30.0)
            && command.style.background_color.as_deref() == Some("#0f1316")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
            && command.style.painter_family == UiPainterFamily::TextField
            && command.style.painter_state == UiPainterResolvedState::Focused
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("build")
            && command.frame == UiFrame::new(60.0, 47.0, 280.0, 16.0)
            && command.style.foreground_color.as_deref() == Some("#e8ecee")
            && command.style.font_size == EditorTypographyTokens::WORKBENCH_BODY_SIZE
            && command.style.line_height == 16.0
            && command.style.painter_family == UiPainterFamily::TextField
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Build Project")
            && command.frame == UiFrame::new(56.0, 84.0, 288.0, 16.0)
            && command.style.foreground_color.as_deref() == Some("#3cc7d6")
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Focused
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Build Assets")
            && command.frame == UiFrame::new(56.0, 112.0, 288.0, 16.0)
            && command.style.foreground_color.as_deref() == Some("#656f76")
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
                && command.frame == UiFrame::new(48.0, 78.0, 304.0, 28.0)
                && command.style.painter_family == UiPainterFamily::PopupRow
        })
        .expect("focused-only command palette row should keep a neutral popup row surface");
    assert_eq!(
        focused_surface.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_surface.style.background_color.as_deref(),
        Some("#141618")
    );
    assert_eq!(
        focused_surface.style.border_color.as_deref(),
        Some("#323a41")
    );
    assert_eq!(focused_surface.style.border_width, 1.0);
    assert!(!commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(48.0, 78.0, 304.0, 28.0)
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.background_color.as_deref() == Some("#2a3036")
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Build Project")
            && command.frame == UiFrame::new(56.0, 84.0, 288.0, 16.0)
            && command.style.foreground_color.as_deref() == Some("#e8ecee")
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Focused
    }));
}

#[test]
fn render_extract_command_palette_honors_valid_visual_and_metric_overrides() {
    let commands = commands_for_command_palette(
        UiFrame::new(40.0, 32.0, 320.0, 160.0),
        r##"
open = true
popup_open = true
query = "build"
commands = ["build_project|label=Build Project"]
background_color = "#111820"
border_color = "#26343d"
search_background_color = "#101820"
search_border_color = "#5ad4df"
foreground_color = "#e1e5e8"
border_width = 2.0
corner_radius = 6.0
search_radius = 3.0
panel_padding_x = 16.0
search_top = 12.0
search_height = 24.0
search_text_inset_x = 6.0
list_gap = 4.0
row_inset_x = 4.0
row_height = 24.0
font_size = 11.0
line_height = 13.0
"##,
    );

    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(40.0, 32.0, 320.0, 160.0)
            && command.style.background_color.as_deref() == Some("#111820")
            && command.style.border_color.as_deref() == Some("#26343d")
            && command.style.border_width == 2.0
            && command.style.corner_radius == 6.0
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(56.0, 44.0, 288.0, 24.0)
            && command.style.background_color.as_deref() == Some("#101820")
            && command.style.border_color.as_deref() == Some("#5ad4df")
            && command.style.border_width == 2.0
            && command.style.corner_radius == 3.0
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("build")
            && command.frame == UiFrame::new(62.0, 49.5, 276.0, 13.0)
            && command.style.foreground_color.as_deref() == Some("#e1e5e8")
            && command.style.font_size == 11.0
            && command.style.line_height == 13.0
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Build Project")
            && command.frame == UiFrame::new(52.0, 76.0, 296.0, 16.0)
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
