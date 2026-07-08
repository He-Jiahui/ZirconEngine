mod control_ids;
mod roles;

use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use control_ids::is_transform_scale_axis_control_id;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum AxisLabelKind {
    Axis(&'static str),
    ScaleLink,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_kind(
    node: &TemplatePaneNodeData,
) -> Option<AxisLabelKind> {
    roles::is_axis_label_role(node.role.as_str())
        .then(|| control_ids::axis_label_kind_from_control_id(node.control_id.as_str()))
        .flatten()
}
