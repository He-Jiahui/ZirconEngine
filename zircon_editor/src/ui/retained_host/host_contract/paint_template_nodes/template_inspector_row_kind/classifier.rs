use super::super::super::data::TemplatePaneNodeData;
use super::constants::{
    COMPONENT_PROPERTY_SLOT_03, COMPONENT_PROPERTY_SLOT_04, COMPONENT_PROPERTY_VIRTUAL_PREFIX,
    INSPECTOR_LIGHTING_ROW, MATERIAL_PROPERTY_ROW, MESH_PROPERTY_ROW,
};
use super::kind::{InspectorResourceKind, InspectorRowKind};
use super::matching::matches_ignore_ascii_case;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn inspector_row_kind(
    node: &TemplatePaneNodeData,
) -> Option<InspectorRowKind> {
    if !is_inspector_property_row(node) {
        return None;
    }

    let label = node.text.trim();
    let value = node.value_text.trim();
    if label.eq_ignore_ascii_case("Lighting") && value.is_empty() {
        return Some(InspectorRowKind::Disclosure);
    }
    if label.eq_ignore_ascii_case("Mesh") && !value.is_empty() {
        return Some(InspectorRowKind::Resource(InspectorResourceKind::Mesh));
    }
    if matches_ignore_ascii_case(label, &["Material", "Materials"]) && !value.is_empty() {
        return Some(InspectorRowKind::Resource(InspectorResourceKind::Material));
    }
    if label.eq_ignore_ascii_case("Cast Shadows") && !value.is_empty() {
        return Some(InspectorRowKind::ShadowSelect);
    }
    if label.eq_ignore_ascii_case("Receive Shadows") && !value.is_empty() {
        return Some(InspectorRowKind::ShadowCheck);
    }
    None
}

fn is_inspector_property_row(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        MESH_PROPERTY_ROW
            | MATERIAL_PROPERTY_ROW
            | COMPONENT_PROPERTY_SLOT_03
            | COMPONENT_PROPERTY_SLOT_04
            | INSPECTOR_LIGHTING_ROW
    ) || node
        .control_id
        .as_str()
        .starts_with(COMPONENT_PROPERTY_VIRTUAL_PREFIX)
}
