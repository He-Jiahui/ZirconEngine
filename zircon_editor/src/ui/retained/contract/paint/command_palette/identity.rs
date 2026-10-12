//! 专用分派的身份门：支持原生组件角色与投影语义角色，让该 painter 只接管命令面板。

use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_command_palette(
    node: &TemplatePaneNodeData,
) -> bool {
    node.role.as_str() == "CommandPalette" || node.component_role.as_str() == "command-palette"
}
