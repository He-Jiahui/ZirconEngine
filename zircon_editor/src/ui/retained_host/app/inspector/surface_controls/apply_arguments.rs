use super::super::super::RetainedEditorHost;
use zircon_runtime_interface::ui::binding::UiBindingValue;

impl RetainedEditorHost {
    pub(super) fn inspector_apply_arguments(&self) -> Result<Vec<UiBindingValue>, String> {
        let Some(inspector) = self.runtime.editor_snapshot().inspector else {
            return Err("Nothing selected".to_string());
        };

        let parent_value = if inspector.parent.trim().is_empty() {
            UiBindingValue::Null
        } else {
            UiBindingValue::string(inspector.parent.clone())
        };
        let mut changes = vec![
            UiBindingValue::array(vec![
                UiBindingValue::string("name"),
                UiBindingValue::string(inspector.name.clone()),
            ]),
            UiBindingValue::array(vec![UiBindingValue::string("parent"), parent_value]),
            UiBindingValue::array(vec![
                UiBindingValue::string("transform.translation.x"),
                UiBindingValue::string(inspector.translation[0].clone()),
            ]),
            UiBindingValue::array(vec![
                UiBindingValue::string("transform.translation.y"),
                UiBindingValue::string(inspector.translation[1].clone()),
            ]),
            UiBindingValue::array(vec![
                UiBindingValue::string("transform.translation.z"),
                UiBindingValue::string(inspector.translation[2].clone()),
            ]),
        ];
        changes.extend(
            inspector
                .plugin_components
                .iter()
                .filter(|component| component.drawer_available)
                .flat_map(|component| component.properties.iter())
                .filter(|property| property.editable)
                .map(|property| {
                    UiBindingValue::array(vec![
                        UiBindingValue::string(property.field_id.clone()),
                        UiBindingValue::string(property.value.clone()),
                    ])
                }),
        );
        let changes = UiBindingValue::array(changes);

        Ok(vec![UiBindingValue::string("entity://selected"), changes])
    }
}
