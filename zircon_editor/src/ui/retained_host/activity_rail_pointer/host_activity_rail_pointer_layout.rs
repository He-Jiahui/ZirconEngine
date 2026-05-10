use zircon_runtime_interface::ui::layout::UiFrame;

use super::host_activity_rail_pointer_item::HostActivityRailPointerItem;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HostActivityRailPointerLayout {
    pub left_strip_frame: UiFrame,
    pub left_tabs: Vec<HostActivityRailPointerItem>,
    pub right_strip_frame: UiFrame,
    pub right_tabs: Vec<HostActivityRailPointerItem>,
}
