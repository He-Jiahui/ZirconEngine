use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::ViewInstanceId;

use super::document_tab_model::DocumentTabModel;

#[derive(Clone, Debug, PartialEq)]
/// 浮窗展示请求；window_id和focused_view用于宿主定位，最终frame须由布局边界确认。
pub struct FloatingWindowModel {
    pub window_id: MainPageId,
    pub title: String,
    pub requested_frame: ShellFrame,
    pub focused_view: Option<ViewInstanceId>,
    pub tabs: Vec<DocumentTabModel>,
}
