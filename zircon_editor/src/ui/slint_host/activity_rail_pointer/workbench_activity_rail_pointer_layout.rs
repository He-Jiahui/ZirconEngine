use zircon_runtime::ui::layout::UiFrame;

use super::workbench_activity_rail_pointer_item::WorkbenchActivityRailPointerItem;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WorkbenchActivityRailPointerLayout {
    pub left_strip_frame: UiFrame,
    pub left_tabs: Vec<WorkbenchActivityRailPointerItem>,
    pub right_strip_frame: UiFrame,
    pub right_tabs: Vec<WorkbenchActivityRailPointerItem>,
}
