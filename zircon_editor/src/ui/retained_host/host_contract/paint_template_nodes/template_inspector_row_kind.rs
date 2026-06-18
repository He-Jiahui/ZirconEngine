use super::super::data::TemplatePaneNodeData;

pub(super) const COMPONENT_PROPERTY_SLOT_03: &str = "WorkbenchComponentPropertySlot03Row";
pub(super) const MATERIAL_PROPERTY_ROW: &str = "WorkbenchMaterialRow";

const COMPONENT_PROPERTY_SLOT_04: &str = "WorkbenchComponentPropertySlot04Row";
const COMPONENT_PROPERTY_VIRTUAL_PREFIX: &str = "WorkbenchComponentPropertyVirtualRow";
const MESH_PROPERTY_ROW: &str = "WorkbenchMeshRow";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InspectorRowKind {
    Resource(InspectorResourceKind),
    Disclosure,
    ShadowSelect,
    ShadowCheck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InspectorResourceKind {
    Mesh,
    Material,
}

pub(super) fn inspector_row_kind(node: &TemplatePaneNodeData) -> Option<InspectorRowKind> {
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

pub(super) fn bool_display_value(value: &str) -> &'static str {
    if bool_value(value) {
        "On"
    } else {
        "Off"
    }
}

pub(super) fn bool_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "on" | "yes" | "check" | "checked"
    )
}

fn is_inspector_property_row(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        MESH_PROPERTY_ROW
            | MATERIAL_PROPERTY_ROW
            | COMPONENT_PROPERTY_SLOT_03
            | COMPONENT_PROPERTY_SLOT_04
            | "WorkbenchInspectorLightingRow"
    ) || node
        .control_id
        .as_str()
        .starts_with(COMPONENT_PROPERTY_VIRTUAL_PREFIX)
}

fn matches_ignore_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}
