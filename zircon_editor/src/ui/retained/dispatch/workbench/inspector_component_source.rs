use crate::ui::workbench::snapshot::{InspectorPluginComponentPropertySnapshot, InspectorSnapshot};

// The existing virtual row payload is reused as a presentation value only.
// Native fields remain in their own snapshot collection and never become plugin components.
pub(super) fn inspector_component_source(
    inspector: &InspectorSnapshot,
) -> (String, Vec<InspectorPluginComponentPropertySnapshot>) {
    let mut labels = Vec::new();
    let mut properties = Vec::new();
    let native_types = inspector
        .native_fields
        .iter()
        .map(|field| field.component_type_path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let multiple = native_types.len() + inspector.plugin_components.len() > 1;
    for field in &inspector.native_fields {
        if !labels.contains(&field.component_label) {
            labels.push(field.component_label.clone());
        }
        properties.push(InspectorPluginComponentPropertySnapshot {
            field_id: field.field_id.clone(),
            name: field.name.clone(),
            label: if multiple {
                format!("{} / {}", field.component_label, field.label)
            } else {
                field.label.clone()
            },
            value: field.value.clone(),
            value_kind: field.value_kind.clone(),
            editable: false,
            field_editor: field.field_editor.clone(),
        });
    }
    for component in &inspector.plugin_components {
        labels.push(component.display_name.clone());
        for source in &component.properties {
            let mut property = source.clone();
            property.editable &= component.customization_available;
            if multiple {
                property.label = format!("{} / {}", component.display_name, property.label);
            }
            properties.push(property);
        }
    }
    let label = match labels.as_slice() {
        [label] => label.clone(),
        [] => String::new(),
        _ => "Components".to_string(),
    };
    (label, properties)
}
