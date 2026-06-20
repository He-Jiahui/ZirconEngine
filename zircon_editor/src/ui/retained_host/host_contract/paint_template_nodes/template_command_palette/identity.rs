use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_command_palette(
    node: &TemplatePaneNodeData,
) -> bool {
    node.role.as_str() == "CommandPalette" || node.component_role.as_str() == "command-palette"
}
