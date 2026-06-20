use super::super::*;
use crate::ui::workbench::snapshot::{MainPageSnapshot, ViewContentKind};

pub(crate) fn asset_surface_visible(
    chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    kind: ViewContentKind,
) -> bool {
    let Some(page) = chrome.workbench.main_pages.iter().find(|page| match page {
        MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => {
            id == &chrome.workbench.active_main_page
        }
    }) else {
        return false;
    };

    match page {
        MainPageSnapshot::Workbench { workspace, .. } => {
            let drawer_visible = chrome.workbench.drawers.values().any(|drawer| {
                drawer.visible
                    && drawer.mode != ActivityDrawerMode::Collapsed
                    && drawer
                        .active_tab
                        .as_ref()
                        .and_then(|active| {
                            drawer.tabs.iter().find(|tab| &tab.instance_id == active)
                        })
                        .or_else(|| drawer.tabs.first())
                        .is_some_and(|tab| tab.content_kind == kind)
            });
            drawer_visible
                || active_workspace_tab(workspace).is_some_and(|tab| tab.content_kind == kind)
        }
        MainPageSnapshot::Exclusive { view, .. } => view.content_kind == kind,
    }
}

pub(super) fn active_workspace_tab(
    workspace: &crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot,
) -> Option<&crate::ui::workbench::snapshot::ViewTabSnapshot> {
    match workspace {
        crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot::Split {
            first, second, ..
        } => active_workspace_tab(first).or_else(|| active_workspace_tab(second)),
        crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot::Tabs { tabs, active_tab } => {
            active_tab
                .as_ref()
                .and_then(|active| tabs.iter().find(|tab| &tab.instance_id == active))
                .or_else(|| tabs.first())
        }
    }
}
