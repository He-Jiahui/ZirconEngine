#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewProjectFormSnapshot {
    pub project_name: String,
    pub location: String,
    pub project_path_preview: String,
    pub template_label: String,
    pub can_create: bool,
    pub can_open_existing: bool,
    pub validation_message: String,
}
