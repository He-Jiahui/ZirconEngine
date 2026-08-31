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
    inspector_row_kind_from_text(label, value)
}

fn inspector_row_kind_from_text(label: &str, value: &str) -> Option<InspectorRowKind> {
    if value.is_empty() {
        return label
            .eq_ignore_ascii_case("Lighting")
            .then_some(InspectorRowKind::Disclosure);
    }
    if label.eq_ignore_ascii_case("Mesh") {
        return Some(InspectorRowKind::Resource(InspectorResourceKind::Mesh));
    }
    if matches_ignore_ascii_case(label, &["Material", "Materials"]) {
        return Some(InspectorRowKind::Resource(InspectorResourceKind::Material));
    }
    if label.eq_ignore_ascii_case("Cast Shadows") {
        return Some(InspectorRowKind::ShadowSelect);
    }
    if label.eq_ignore_ascii_case("Receive Shadows") {
        return Some(InspectorRowKind::ShadowCheck);
    }
    None
}

#[cfg(test)]
#[path = "classifier/empty_value_short_circuit_tests.rs"]
mod empty_value_short_circuit_tests;

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
