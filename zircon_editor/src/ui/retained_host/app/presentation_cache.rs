use crate::ui::workbench::snapshot::{EditorChromeSnapshot, WorkbenchSnapshot};

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

    pub(super) fn welcome_recent_project_paths(&self) -> &[String] {
        &self.welcome_recent_project_paths
    }

    pub(super) fn console_status_line(&self) -> &str {
        &self.console_status_line
    }
}
