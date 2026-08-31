use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HostPagePointerItem {
    pub page_id: MainPageId,
    pub close_instance_id: Option<ViewInstanceId>,
}
