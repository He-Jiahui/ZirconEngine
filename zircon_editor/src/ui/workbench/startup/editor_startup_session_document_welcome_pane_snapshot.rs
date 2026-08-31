use super::display_project_path::display_project_path;
use super::editor_startup_session_document::EditorStartupSessionDocument;
use super::format_recent_project_time::format_recent_project_time;
use super::new_project_form_snapshot::NewProjectFormSnapshot;
use super::now_unix_ms::now_unix_ms;
use super::recent_project_item_snapshot::RecentProjectItemSnapshot;
use super::welcome_pane_snapshot::WelcomePaneSnapshot;

impl EditorStartupSessionDocument {
    pub fn welcome_pane_snapshot(&self, browse_supported: bool) -> WelcomePaneSnapshot {
        let project_path_preview = self
            .draft
            .project_root()
            .map(|path| display_project_path(path.to_string_lossy()))
            .unwrap_or_default();
        let now_unix_ms = now_unix_ms();

        WelcomePaneSnapshot {
            title: "Open or Create".to_string(),
            subtitle: "Recent projects".to_string(),
            status_message: self.status_message.clone(),
            browse_supported,
            recent_projects: self
                .recent_projects
                .iter()
                .enumerate()
                .map(|(index, entry)| RecentProjectItemSnapshot {
                    display_name: entry.summary.name.clone(),
                    path: display_project_path(&entry.path),
                    validation: entry.validation,
                    last_opened_label: format_recent_project_time(
                        entry.last_opened_unix_ms,
                        now_unix_ms,
                    ),
                    selected: index == 0,
                })
                .collect(),
            form: NewProjectFormSnapshot {
                project_name: self.draft.project_name.clone(),
                location: self.draft.location.clone(),
                project_path_preview,
                template_label: "Renderable Empty".to_string(),
                can_create: self.creation_validation.is_empty(),
                can_open_existing: self.can_open_existing,
                validation_message: self.creation_validation.clone(),
            },
        }
    }
}
