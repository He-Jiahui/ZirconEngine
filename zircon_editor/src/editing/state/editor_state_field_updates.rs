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
