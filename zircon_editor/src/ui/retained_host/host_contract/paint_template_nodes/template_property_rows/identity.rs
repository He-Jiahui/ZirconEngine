use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const COMPONENT_PROPERTY_SLOT_03: &str = "WorkbenchComponentPropertySlot03Row";
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const COMPONENT_PROPERTY_SLOT_04: &str = "WorkbenchComponentPropertySlot04Row";
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const COMPONENT_PROPERTY_VIRTUAL_PREFIX: &str = "WorkbenchComponentPropertyVirtualRow";
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MESH_PROPERTY_ROW:
    &str = "WorkbenchMeshRow";
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MATERIAL_PROPERTY_ROW: &str = "WorkbenchMaterialRow";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_property_row(
    node: &TemplatePaneNodeData,
) -> bool {
    is_component_property_row(node)
        || node.component_role.as_str() == "property-row"
        || node.role.as_str() == "PropertyRow"
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_component_property_row(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.control_id.as_str(),
        MESH_PROPERTY_ROW
            | MATERIAL_PROPERTY_ROW
            | COMPONENT_PROPERTY_SLOT_03
            | COMPONENT_PROPERTY_SLOT_04
    ) || node
        .control_id
        .as_str()
        .starts_with(COMPONENT_PROPERTY_VIRTUAL_PREFIX)
}
