use crate::snapshot::{EditorChromeSnapshot, MainPageSnapshot};

use super::super::host_page_tab_model::HostPageTabModel;
use super::super::main_host_strip_model::MainHostStripModel;
use super::super::main_host_strip_view_model::MainHostStripViewModel;
use super::breadcrumbs::breadcrumbs_for_page;
use super::page_access::{page_dirty, page_id, page_title};

pub(crate) fn host_strip_model(
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
