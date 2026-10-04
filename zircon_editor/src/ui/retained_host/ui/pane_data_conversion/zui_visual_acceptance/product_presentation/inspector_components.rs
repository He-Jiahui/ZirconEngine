use super::{InspectorComponentSnapshot, InspectorPropertySnapshot};
use crate::ui::workbench::snapshot::InspectorSnapshot;

/// Neutral serialized components carry the same selected-world fields as the normal bridge.
/// Native component identities remain separate from plugin ownership and are read-only.
pub(super) fn project_inspector_components(
    inspector: &InspectorSnapshot,
) -> Vec<InspectorComponentSnapshot> {
    let mut components: Vec<InspectorComponentSnapshot> = Vec::new();
    for field in &inspector.native_fields {
        let index = match components
            .iter()
            .position(|component| component.id == field.component_type_path)
        {
            Some(index) => index,
            None => {
                components.push(InspectorComponentSnapshot {
                    id: field.component_type_path.clone(),
                    title: field.component_label.clone(),
                    properties: Vec::new(),
                });
                components.len() - 1
            }
        };
        components[index]
            .properties
            .push(InspectorPropertySnapshot {
                id: field.field_id.clone(),
                label: field.label.clone(),
                value: field.value.clone(),
                kind: field.value_kind.clone(),
                editable: false,
            });
    }
    components.extend(inspector.plugin_components.iter().map(|component| {
        InspectorComponentSnapshot {
            id: component.component_id.clone(),
            title: component.display_name.clone(),
            properties: component
                .properties
                .iter()
                .map(|property| InspectorPropertySnapshot {
                    id: property.field_id.clone(),
                    label: property.label.clone(),
                    value: property.value.clone(),
                    kind: property.value_kind.clone(),
                    editable: property.editable && component.customization_available,
                })
                .collect(),
        }
    }));
    components
}

#[cfg(test)]
#[path = "inspector_components/tests/cases.rs"]
mod tests;
