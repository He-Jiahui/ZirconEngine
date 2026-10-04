use super::*;
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, MainPageSnapshot, WorkbenchSnapshot};

impl RetainedEditorHost {
    #[cfg(test)]
    pub(super) fn active_activity_window_template_document_id(&self) -> Option<String> {
        let chrome = self.runtime.chrome_snapshot();
        active_activity_window_template_document_id(&chrome).map(str::to_string)
    }

    pub(super) fn active_activity_window_template_document_is(&self, document_id: &str) -> bool {
        self.runtime
            .active_activity_window_template_document_is(document_id)
    }
}

pub(super) fn active_activity_window_template_document_id(
    chrome: &EditorChromeSnapshot,
) -> Option<&str> {
    active_activity_window_template_document_id_from_workbench(&chrome.workbench)
}

pub(super) fn active_activity_window_template_document_id_from_workbench(
    workbench: &WorkbenchSnapshot,
) -> Option<&str> {
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

pub(super) fn welcome_recent_project_paths(chrome: &EditorChromeSnapshot) -> Vec<String> {
    chrome
        .welcome
        .recent_projects
        .iter()
        .map(|recent| recent.path.clone())
        .collect()
}
