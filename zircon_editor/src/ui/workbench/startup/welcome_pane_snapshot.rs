use super::new_project_form_snapshot::NewProjectFormSnapshot;
use super::recent_project_item_snapshot::RecentProjectItemSnapshot;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WelcomePaneSnapshot {
    pub title: String,
    pub subtitle: String,
    pub status_message: String,
    pub browse_supported: bool,
    pub recent_projects: Vec<RecentProjectItemSnapshot>,
    pub form: NewProjectFormSnapshot,
}
