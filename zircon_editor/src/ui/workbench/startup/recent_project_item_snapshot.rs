use super::recent_project_validation::RecentProjectValidation;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecentProjectItemSnapshot {
    pub display_name: String,
    pub path: String,
    pub validation: RecentProjectValidation,
    pub last_opened_label: String,
    pub selected: bool,
}
