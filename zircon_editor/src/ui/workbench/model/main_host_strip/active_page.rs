use crate::snapshot::{DocumentWorkspaceSnapshot, EditorChromeSnapshot, MainPageSnapshot};
use crate::MainPageId;

use super::page_access::page_id;

pub(crate) fn active_page_snapshot(chrome: &EditorChromeSnapshot) -> MainPageSnapshot {
    chrome
        .workbench
        .main_pages
        .iter()
        .find(|page| page_id(page) == &chrome.workbench.active_main_page)
        .cloned()
        .unwrap_or_else(|| {
            chrome
                .workbench
                .main_pages
                .first()
                .cloned()
                .unwrap_or_else(|| MainPageSnapshot::Workbench {
                    id: MainPageId::workbench(),
                    title: "Workbench".to_string(),
                    workspace: DocumentWorkspaceSnapshot::Tabs {
                        tabs: Vec::new(),
                        active_tab: None,
                    },
                })
        })
}
