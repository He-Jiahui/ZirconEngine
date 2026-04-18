use super::recent_project_entry::RecentProjectEntry;
use super::recent_project_validation::RecentProjectValidation;
use super::stored_startup_session::StoredStartupSession;

impl StoredStartupSession {
    pub fn recent_projects_with_validation<F>(&self, mut validate: F) -> Vec<RecentProjectEntry>
    where
        F: FnMut(&str) -> RecentProjectValidation,
    {
        self.recent_projects
            .iter()
            .map(|entry| RecentProjectEntry {
                display_name: entry.display_name.clone(),
                path: entry.path.clone(),
                last_opened_unix_ms: entry.last_opened_unix_ms,
                validation: validate(&entry.path),
            })
            .collect()
    }
}
