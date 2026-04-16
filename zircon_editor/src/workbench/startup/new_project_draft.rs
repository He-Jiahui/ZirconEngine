use serde::{Deserialize, Serialize};

use super::new_project_template::NewProjectTemplate;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewProjectDraft {
    pub project_name: String,
    pub location: String,
    pub template: NewProjectTemplate,
}
