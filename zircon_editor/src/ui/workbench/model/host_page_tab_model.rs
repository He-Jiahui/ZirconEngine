use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostPageTabModel {
    pub id: MainPageId,
    pub title: String,
    pub dirty: bool,
    pub closeable: bool,
    pub close_instance_id: Option<ViewInstanceId>,
}
