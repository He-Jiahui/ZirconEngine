use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;

use super::super::workbench_command::{workbench_command_visual_role, WorkbenchCommandVisualRole};
use super::model::WorkbenchButtonStyle;

pub(super) fn is_prominent_workbench_command_button(node: &TemplatePaneNodeData) -> bool {
    !matches!(
        workbench_command_visual_role(node),
        WorkbenchCommandVisualRole::None
    )
}

pub(super) fn prominent_workbench_command_style(
    node: &TemplatePaneNodeData,
    style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    match workbench_command_visual_role(node) {
        WorkbenchCommandVisualRole::PrimaryImport => {
            primary_import_workbench_command_style(node, style)
        }
        WorkbenchCommandVisualRole::MutedProminent => {
            muted_prominent_workbench_command_style(node, style)
        }
        WorkbenchCommandVisualRole::None => style,
    }
}

fn muted_prominent_workbench_command_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    let active = node.selected || node.checked || node.focused || node.popup_open;
    let palette = current_host_palette();
    style.surface = if node.pressed {
        palette.surface
    } else if active || node.hovered {
        palette.surface_hover
    } else {
        palette.surface_pressed
    };
    style.border = palette.border;
    style.border_width = 1.0;
    style.text = palette.accent;
    style.glyph = palette.accent;
    style
}

fn primary_import_workbench_command_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    let active = node.selected || node.checked || node.focused || node.popup_open;
    let palette = current_host_palette();
    let surface = if node.pressed || active || node.hovered {
        palette.focus_ring
    } else {
        palette.accent
    };
    style.surface = surface;
    style.border = surface;
    style.border_width = 1.0;
    style.text = palette.shell_background;
    style.glyph = palette.shell_background;
    style
}
