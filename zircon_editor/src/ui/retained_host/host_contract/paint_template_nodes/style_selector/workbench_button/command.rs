use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::model::WorkbenchButtonStyle;

pub(super) fn is_prominent_workbench_command_button(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        "ImportModel"
            | "WorkbenchAssetsImportButton"
            | "WorkbenchModuleCompile"
            | "WorkbenchToolbarCompile"
    ) || matches!(
        node.action_id.as_str(),
        "workbench.asset.import_model"
            | "workbench.module.assets.import.invoke"
            | "workbench.module.assets.import_now"
            | "workbench.module.compile"
            | "workbench.toolbar.compile"
    )
}

pub(super) fn prominent_workbench_command_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    let active = node.selected || node.checked || node.focused || node.popup_open;
    style.surface = if node.pressed {
        PALETTE.surface
    } else if active || node.hovered {
        PALETTE.surface_hover
    } else {
        PALETTE.surface_pressed
    };
    style.border = PALETTE.border;
    style.border_width = 1.0;
    style.text = PALETTE.accent;
    style.glyph = PALETTE.accent;
    style
}
