use super::editor_state::EditorState;
use zircon_runtime_interface::reflect::{ReflectObjectAddress, ReflectReadRequest};

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
        let Some((component_type_path, field_name)) = field_id.rsplit_once('.') else {
            return false;
        };
        let Some(selected) = self.viewport_controller.selected_node() else {
            return false;
        };
        self.world
            .try_with_world(|scene| {
                let Ok(schema) = scene.reflect_schema(component_type_path) else {
                    return false;
                };
                let field_editable = schema.type_info.fields.iter().any(|field| {
                    field.name == field_name && field.editor_visible && field.editable
                });
                if !field_editable {
                    return false;
                }

                let Ok(address) = ReflectObjectAddress::component(selected, component_type_path)
                else {
                    return false;
                };
                scene
                    .reflect_read(ReflectReadRequest::new(address, field_name))
                    .is_ok()
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
