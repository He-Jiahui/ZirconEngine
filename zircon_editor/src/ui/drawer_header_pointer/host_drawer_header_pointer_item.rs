use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HostDrawerHeaderPointerItem {
    pub slot: ActivityDrawerSlot,
    pub instance_id: ViewInstanceId,
}
