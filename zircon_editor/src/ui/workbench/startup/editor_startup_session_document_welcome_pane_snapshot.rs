use super::editor_startup_session_document::EditorStartupSessionDocument;
use super::format_recent_project_time::format_recent_project_time;
use super::new_project_form_snapshot::NewProjectFormSnapshot;
use super::recent_project_item_snapshot::RecentProjectItemSnapshot;
use super::welcome_pane_snapshot::WelcomePaneSnapshot;

impl EditorStartupSessionDocument {
    pub fn welcome_pane_snapshot(&self, browse_supported: bool) -> WelcomePaneSnapshot {
        let project_path_preview = self
            .draft
            .project_root()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default();
        let creation_validation = self
            .draft
            .validate_for_creation()
            .map(|_| String::new())
            .unwrap_or_else(|error| error);
        let can_open_existing = self.draft.validate_for_open_existing().is_ok();

        WelcomePaneSnapshot {
            title: "Open or Create".to_string(),
            subtitle: "Continue from a recent project or scaffold a renderable empty project."
                .to_string(),
            status_message: self.status_message.clone(),
            browse_supported,
            recent_projects: self
                .recent_projects
                .iter()
                .enumerate()
                .map(|(index, entry)| RecentProjectItemSnapshot {
                    display_name: entry.display_name.clone(),
                    path: entry.path.clone(),
                    validation: entry.validation,
                    last_opened_label: format_recent_project_time(entry.last_opened_unix_ms),
                    selected: index == 0,
                })
                .collect(),
            form: NewProjectFormSnapshot {
                project_name: self.draft.project_name.clone(),
                location: self.draft.location.clone(),
                project_path_preview,
                template_label: "Renderable Empty".to_string(),
                can_create: creation_validation.is_empty(),
                can_open_existing,
                validation_message: creation_validation,
            },
        }
    }
}
