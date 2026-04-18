use super::stored_startup_session::StoredStartupSession;

impl StoredStartupSession {
    pub fn forget_recent_project(&mut self, path: &str) {
        self.recent_projects.retain(|entry| entry.path != path);
        if self.last_project_path.as_deref() == Some(path) {
            self.last_project_path = self.recent_projects.first().map(|entry| entry.path.clone());
        }
    }
}
