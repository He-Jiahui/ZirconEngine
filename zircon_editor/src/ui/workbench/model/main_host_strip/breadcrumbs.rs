use crate::ui::workbench::snapshot::{EditorChromeSnapshot, MainPageSnapshot, ViewContentKind};

use super::super::breadcrumb_model::BreadcrumbModel;
use super::active_view::active_view_in_workspace;

/// 页头显示当前页面和内容线索；普通split树使用首个可用活动标签，不代表全局焦点。
pub(super) fn breadcrumbs_for_page(
    page: &MainPageSnapshot,
    chrome: &EditorChromeSnapshot,
) -> Vec<BreadcrumbModel> {
    match page {
        MainPageSnapshot::Workbench {
            title, workspace, ..
        } => {
            let mut breadcrumbs = breadcrumb_buffer(title.clone());
            if let Some(active_view) = active_view_in_workspace(workspace) {
                breadcrumbs.push(BreadcrumbModel {
                    label: active_view.title.clone(),
                });
            }
            breadcrumbs
        }
        MainPageSnapshot::Exclusive { title, view, .. } => {
            let mut breadcrumbs = breadcrumb_buffer(title.clone());
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

fn breadcrumb_buffer(label: String) -> Vec<BreadcrumbModel> {
    let mut breadcrumbs = Vec::with_capacity(2);
    breadcrumbs.push(BreadcrumbModel { label });
    breadcrumbs
}

#[cfg(test)]
#[path = "tests/breadcrumbs_optimization_tests.rs"]
mod optimization_tests;
