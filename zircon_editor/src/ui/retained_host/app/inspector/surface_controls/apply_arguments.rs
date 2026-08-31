use super::super::super::RetainedEditorHost;
use crate::ui::workbench::snapshot::InspectorSnapshot;
use zircon_runtime_interface::ui::binding::UiBindingValue;

impl RetainedEditorHost {
    pub(super) fn inspector_apply_arguments(&self) -> Result<Vec<UiBindingValue>, String> {
        let Some(inspector) = self.runtime.editor_snapshot().inspector else {
            return Err("Nothing selected".to_string());
        };

        Ok(inspector_apply_arguments_from_snapshot(inspector))
    }
}

fn inspector_apply_arguments_from_snapshot(inspector: InspectorSnapshot) -> Vec<UiBindingValue> {
    let parent_value = if inspector.parent.trim().is_empty() {
        UiBindingValue::Null
    } else {
        UiBindingValue::string(inspector.parent)
    };
    let [translation_x, translation_y, translation_z] = inspector.translation;
    let mut changes = vec![
        UiBindingValue::array(vec![
            UiBindingValue::string("name"),
            UiBindingValue::string(inspector.name),
        ]),
        UiBindingValue::array(vec![UiBindingValue::string("parent"), parent_value]),
        UiBindingValue::array(vec![
            UiBindingValue::string("transform.translation.x"),
            UiBindingValue::string(translation_x),
        ]),
        UiBindingValue::array(vec![
            UiBindingValue::string("transform.translation.y"),
            UiBindingValue::string(translation_y),
        ]),
        UiBindingValue::array(vec![
            UiBindingValue::string("transform.translation.z"),
            UiBindingValue::string(translation_z),
        ]),
    ];
    changes.extend(
        inspector
            .plugin_components
            .into_iter()
            .filter(|component| component.customization_available)
            .flat_map(|component| component.properties.into_iter())
            .filter(|property| property.editable)
            .map(|property| {
                UiBindingValue::array(vec![
                    UiBindingValue::string(property.field_id),
                    UiBindingValue::string(property.value),
                ])
            }),
    );
    let changes = UiBindingValue::array(changes);

    vec![UiBindingValue::string("entity://selected"), changes]
}

#[cfg(test)]
#[path = "apply_arguments/owned_snapshot_tests.rs"]
mod owned_snapshot_tests;
