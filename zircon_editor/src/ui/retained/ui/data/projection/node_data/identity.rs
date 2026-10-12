use crate::ui::retained_host as host_contract;

pub(super) fn assign_identity_fields(
    node: &mut host_contract::TemplatePaneNodeData,
    node_id: String,
    control_id: String,
    role: String,
    component_role: String,
) {
    node.node_id = node_id.into();
    node.control_id = control_id.into();
    node.role = role.into();
    node.component_role = component_role.into();
}
