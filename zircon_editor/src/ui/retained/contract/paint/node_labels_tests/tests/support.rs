use super::super::super::super::data::TemplatePaneNodeData;

pub(super) fn property_row_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        role: "Mount".into(),
        component_role: "property-row".into(),
        text: "Position".into(),
        value_text: "X 12.0   Y 3.5   Z -8.0".into(),
        ..TemplatePaneNodeData::default()
    }
}
