use crate::layout::MainPageId;

use super::breadcrumb_model::BreadcrumbModel;
use super::host_page_tab_model::HostPageTabModel;
use super::main_host_strip_model::MainHostStripModel;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MainHostStripViewModel {
    pub mode: MainHostStripModel,
    pub pages: Vec<HostPageTabModel>,
    pub active_page: MainPageId,
    pub breadcrumbs: Vec<BreadcrumbModel>,
}
