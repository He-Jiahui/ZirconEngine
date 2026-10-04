use crate::ui::workbench::layout::MainPageId;

use super::breadcrumb_model::BreadcrumbModel;
use super::host_page_tab_model::HostPageTabModel;
use super::main_host_strip_model::MainHostStripModel;

#[derive(Clone, Debug, PartialEq, Eq)]
/// 主页面导航投影；活动页身份和宿主模式用于选择内容，路径文字只用于展示。
pub struct MainHostStripViewModel {
    pub mode: MainHostStripModel,
    pub pages: Vec<HostPageTabModel>,
    pub active_page: MainPageId,
    pub breadcrumbs: Vec<BreadcrumbModel>,
}
