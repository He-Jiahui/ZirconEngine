use crate::snapshot::{
    DocumentWorkspaceSnapshot, EditorChromeSnapshot, MainPageSnapshot, ViewContentKind,
    ViewTabSnapshot,
};
use crate::MainPageId;

use super::breadcrumb_model::BreadcrumbModel;
use super::host_page_tab_model::HostPageTabModel;
use super::main_host_strip_model::MainHostStripModel;
use super::main_host_strip_view_model::MainHostStripViewModel;

pub(super) fn active_page_snapshot(chrome: &EditorChromeSnapshot) -> MainPageSnapshot {
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

pub(super) fn host_strip_model(
    active_page: &MainPageSnapshot,
    chrome: &EditorChromeSnapshot,
) -> MainHostStripViewModel {
    MainHostStripViewModel {
        mode: match active_page {
            MainPageSnapshot::Workbench { .. } => MainHostStripModel::Workbench,
            MainPageSnapshot::Exclusive { view, .. } => MainHostStripModel::ExclusiveWindow {
                instance_id: view.instance_id.clone(),
            },
        },
        pages: chrome
            .workbench
            .main_pages
            .iter()
            .map(|page| HostPageTabModel {
                id: page_id(page).clone(),
                title: page_title(page).to_string(),
                dirty: page_dirty(page),
                closeable: matches!(page, MainPageSnapshot::Exclusive { .. }),
            })
            .collect(),
        active_page: page_id(active_page).clone(),
        breadcrumbs: breadcrumbs_for_page(active_page, chrome),
    }
}

fn breadcrumbs_for_page(
    page: &MainPageSnapshot,
    chrome: &EditorChromeSnapshot,
) -> Vec<BreadcrumbModel> {
    match page {
        MainPageSnapshot::Workbench {
            title, workspace, ..
        } => {
            let mut breadcrumbs = vec![BreadcrumbModel {
                label: title.clone(),
            }];
            if let Some(active_view) = active_view_in_workspace(workspace) {
                breadcrumbs.push(BreadcrumbModel {
                    label: active_view.title.clone(),
                });
            }
            breadcrumbs
        }
        MainPageSnapshot::Exclusive { title, view, .. } => {
            let mut breadcrumbs = vec![BreadcrumbModel {
                label: title.clone(),
            }];
            if view.content_kind == ViewContentKind::Welcome {
                breadcrumbs.push(BreadcrumbModel {
                    label: chrome.welcome.title.clone(),
                });
            } else if let Some(path) = view
                .serializable_payload
                .get("path")
                .and_then(|value| value.as_str())
            {
                breadcrumbs.push(BreadcrumbModel {
                    label: path.to_string(),
                });
            } else {
                breadcrumbs.push(BreadcrumbModel {
                    label: view.title.clone(),
                });
            }
            breadcrumbs
        }
    }
}

fn active_view_in_workspace(workspace: &DocumentWorkspaceSnapshot) -> Option<&ViewTabSnapshot> {
    match workspace {
        DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            active_view_in_workspace(first).or_else(|| active_view_in_workspace(second))
        }
        DocumentWorkspaceSnapshot::Tabs { tabs, active_tab } => active_tab
            .as_ref()
            .and_then(|active_id| tabs.iter().find(|tab| &tab.instance_id == active_id))
            .or_else(|| tabs.first()),
    }
}

fn page_id(page: &MainPageSnapshot) -> &MainPageId {
    match page {
        MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => id,
    }
}

fn page_title(page: &MainPageSnapshot) -> &str {
    match page {
        MainPageSnapshot::Workbench { title, .. } | MainPageSnapshot::Exclusive { title, .. } => {
            title
        }
    }
}

fn page_dirty(page: &MainPageSnapshot) -> bool {
    match page {
        MainPageSnapshot::Workbench { workspace, .. } => active_view_in_workspace(workspace)
            .map(|view| view.dirty)
            .unwrap_or(false),
        MainPageSnapshot::Exclusive { view, .. } => view.dirty,
    }
}
