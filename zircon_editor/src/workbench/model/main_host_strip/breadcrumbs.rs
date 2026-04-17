use crate::snapshot::{EditorChromeSnapshot, MainPageSnapshot, ViewContentKind};

use super::super::breadcrumb_model::BreadcrumbModel;
use super::active_view::active_view_in_workspace;

pub(super) fn breadcrumbs_for_page(
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
