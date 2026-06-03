use crate::ui::workbench::snapshot::{EditorChromeSnapshot, MainPageSnapshot, WorkbenchSnapshot};

#[derive(Clone, Debug, Default)]
pub(super) struct HostPresentationCache {
    workbench: Option<WorkbenchSnapshot>,
    welcome_recent_project_paths: Vec<String>,
    console_status_line: String,
}

impl HostPresentationCache {
    pub(super) fn update_from_chrome(&mut self, chrome: &EditorChromeSnapshot) {
        self.workbench = Some(chrome.workbench.clone());
        self.welcome_recent_project_paths = chrome
            .welcome
            .recent_projects
            .iter()
            .map(|recent| recent.path.clone())
            .collect();
        self.console_status_line = chrome.status_line.clone();
    }

    pub(super) fn workbench(&self) -> Option<&WorkbenchSnapshot> {
        self.workbench.as_ref()
    }

    pub(super) fn active_activity_window_template_document_id(&self) -> Option<&str> {
        let workbench = self.workbench.as_ref()?;
        let active_page = workbench.main_pages.iter().find(|page| match page {
            MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => {
                id == &workbench.active_main_page
            }
        })?;
        match active_page {
            MainPageSnapshot::Workbench {
                activity_window_template,
                ..
            } => activity_window_template
                .as_ref()
                .map(|template| template.document_id.as_str()),
            MainPageSnapshot::Exclusive { view, .. } => view
                .activity_window_template
                .as_ref()
                .map(|template| template.document_id.as_str()),
        }
    }

    pub(super) fn welcome_recent_project_paths(&self) -> &[String] {
        &self.welcome_recent_project_paths
    }

    pub(super) fn console_status_line(&self) -> &str {
        &self.console_status_line
    }
}
