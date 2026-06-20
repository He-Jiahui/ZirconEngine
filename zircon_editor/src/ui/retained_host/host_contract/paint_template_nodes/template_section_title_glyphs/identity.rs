use super::super::super::data::TemplatePaneNodeData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum SectionTitleIcon {
    Cube,
    Transform,
    Mesh,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_title_icon(
    node: &TemplatePaneNodeData,
) -> Option<SectionTitleIcon> {
    match node.control_id.as_str() {
        "WorkbenchInspectorTitle" => Some(SectionTitleIcon::Cube),
        "WorkbenchTransformLabel" => Some(SectionTitleIcon::Transform),
        "WorkbenchMeshLabel" => Some(SectionTitleIcon::Mesh),
        _ => None,
    }
}
