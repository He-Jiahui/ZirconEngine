use super::editor_state::EditorState;

impl EditorState {
    pub fn update_translation_field(&mut self, axis: usize, value: String) -> bool {
        self.transform_fields[axis] = value;
        false
    }

    pub fn update_name_field(&mut self, value: String) {
        self.name_field = value;
    }

    pub fn update_parent_field(&mut self, value: String) {
        self.parent_field = value;
    }

    pub fn update_dynamic_component_field(&mut self, field_id: impl Into<String>, value: String) {
        self.inspector_dynamic_fields.insert(field_id.into(), value);
    }

    pub(crate) fn can_edit_dynamic_component_field(&self, field_id: &str) -> bool {
        let Some((component_id, property)) = field_id.rsplit_once('.') else {
            return false;
        };
        let Some(selected) = self.viewport_controller.selected_node() else {
            return false;
        };
        self.world
            .try_with_world(|scene| {
                let Some(descriptor) = scene.component_type_descriptor(component_id) else {
                    return false;
                };
                if scene.dynamic_component(selected, component_id).is_none() {
                    return false;
                }
                descriptor.properties.is_empty()
                    || descriptor
                        .properties
                        .iter()
                        .any(|candidate| candidate.name == property && candidate.editable)
            })
            .unwrap_or(false)
    }

    pub fn set_mesh_import_path(&mut self, value: String) {
        self.mesh_import_path = value;
    }

    pub fn set_project_path(&mut self, value: String) {
        self.project_path = value;
    }

    pub fn set_status_line(&mut self, value: impl Into<String>) {
        self.status_line = value.into();
    }
}
