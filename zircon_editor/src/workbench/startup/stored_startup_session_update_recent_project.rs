use super::constants::RECENT_PROJECT_LIMIT;
use super::stored_recent_project_entry::StoredRecentProjectEntry;
use super::stored_startup_session::StoredStartupSession;

impl StoredStartupSession {
    pub fn update_recent_project(&mut self, path: &str, display_name: &str, now_unix_ms: u64) {
        self.last_project_path = Some(path.to_string());
        self.recent_projects.retain(|entry| entry.path != path);
        self.recent_projects.insert(
            0,
            StoredRecentProjectEntry {
                display_name: display_name.to_string(),
                path: path.to_string(),
                last_opened_unix_ms: now_unix_ms,
            },
        );
        self.recent_projects.sort_by(|left, right| {
            right
                .last_opened_unix_ms
                .cmp(&left.last_opened_unix_ms)
                .then_with(|| left.path.cmp(&right.path))
        });
        self.recent_projects.truncate(RECENT_PROJECT_LIMIT);
    }
}
