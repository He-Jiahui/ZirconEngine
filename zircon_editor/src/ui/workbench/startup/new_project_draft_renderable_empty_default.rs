use super::default_project_location::default_project_location;
use super::new_project_draft::NewProjectDraft;
use super::new_project_template::NewProjectTemplate;

impl NewProjectDraft {
    pub fn renderable_empty_default() -> Self {
        Self {
            project_name: "ZirconProject".to_string(),
            location: default_project_location().to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        }
    }
}
