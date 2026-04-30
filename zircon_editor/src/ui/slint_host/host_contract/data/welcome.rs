use slint::{ModelRc, SharedString};

use super::TemplatePaneNodeData;

#[derive(Clone, Default)]
pub(crate) struct RecentProjectData {
    pub display_name: SharedString,
    pub path: SharedString,
    pub last_opened_label: SharedString,
    pub status_label: SharedString,
    pub invalid: bool,
}

#[derive(Clone, Default)]
pub(crate) struct NewProjectFormData {
    pub project_name: SharedString,
    pub location: SharedString,
    pub project_path_preview: SharedString,
    pub template_label: SharedString,
    pub validation_message: SharedString,
    pub can_create: bool,
    pub can_open_existing: bool,
    pub browse_supported: bool,
}

#[derive(Clone, Default)]
pub(crate) struct WelcomePaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub title: SharedString,
    pub subtitle: SharedString,
    pub status_message: SharedString,
    pub form: NewProjectFormData,
}
