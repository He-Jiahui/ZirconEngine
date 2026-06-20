use super::super::super::data::TemplatePaneNodeData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum AxisLabelKind {
    Axis(&'static str),
    ScaleLink,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_kind(
    node: &TemplatePaneNodeData,
) -> Option<AxisLabelKind> {
    if !matches!(node.role.as_str(), "Label" | "Icon" | "SvgIcon") {
        return None;
    }
    let control_id = node.control_id.as_str();
    if control_id == "WorkbenchTransformScaleLink" {
        return Some(AxisLabelKind::ScaleLink);
    }
    transform_axis_label(control_id).map(AxisLabelKind::Axis)
}

fn transform_axis_label(control_id: &str) -> Option<&'static str> {
    let field = control_id.strip_prefix("WorkbenchTransform")?;
    if field.ends_with("AxisX") {
        Some("X")
    } else if field.ends_with("AxisY") {
        Some("Y")
    } else if field.ends_with("AxisZ") {
        Some("Z")
    } else {
        None
    }
}
