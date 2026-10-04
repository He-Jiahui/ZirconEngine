//! 导入与编译命令在普通按钮配方之后应用独立视觉角色；不可用状态已由上层选择器提前返回。

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

use super::super::resolved_state_for_node;
use super::super::workbench_command::{workbench_command_visual_role, WorkbenchCommandVisualRole};
use super::metrics::workbench_button_border_width;
use super::model::WorkbenchButtonStyle;
use super::palette::workbench_button_command_palette;

pub(super) fn is_prominent_workbench_command_button(node: &TemplatePaneNodeData) -> bool {
    !matches!(
        workbench_command_visual_role(node),
        WorkbenchCommandVisualRole::None
    )
}

// TODO: [CR-EDITOR-PAINT-STYLE-0002] 确认命令特例是否应沿用合成的按压态；当前仅读 node.pressed，会覆盖 enter_pressed 或声明 Pressed 的表面，补同状态等价测试。
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
    let active = node.selected || node.checked || node.popup_open;
    let focus_visible = resolved_state_for_node(node).focus_visible;
    let command_palette = workbench_button_command_palette();
    style.surface = if node.pressed {
        command_palette.muted_pressed_surface
    } else if active || node.hovered {
        command_palette.muted_hot_surface
    } else {
        command_palette.muted_rest_surface
    };
    style.border = if focus_visible && !node.pressed {
        style.border
    } else {
        command_palette.muted_border
    };
    style.border_width = workbench_button_border_width();
    style.text = command_palette.muted_text;
    style.glyph = command_palette.muted_text;
    style
}

fn primary_import_workbench_command_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    let active = node.selected || node.checked || node.popup_open;
    let focus_visible = resolved_state_for_node(node).focus_visible;
    let command_palette = workbench_button_command_palette();
    let surface = if node.pressed {
        command_palette.primary_pressed_surface
    } else if active || node.hovered {
        command_palette.primary_hot_surface
    } else {
        command_palette.primary_rest_surface
    };
    style.surface = surface;
    style.border = if focus_visible && !node.pressed {
        style.border
    } else {
        surface
    };
    style.border_width = workbench_button_border_width();
    let text = if node.pressed {
        command_palette.primary_pressed_text
    } else {
        command_palette.primary_text
    };
    style.text = text;
    style.glyph = text;
    style
}
