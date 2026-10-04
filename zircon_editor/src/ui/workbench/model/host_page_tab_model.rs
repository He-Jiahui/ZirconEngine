use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, PartialEq, Eq)]
/// 主页面标签；独占页关闭目标是其视图实例，不能把页面ID当作关闭视图ID。
pub struct HostPageTabModel {
    pub id: MainPageId,
    pub title: String,
    pub dirty: bool,
    pub closeable: bool,
    pub close_instance_id: Option<ViewInstanceId>,
}
