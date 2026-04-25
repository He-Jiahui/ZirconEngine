use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, event_ui::UiNodeId, surface::UiSurface};

use super::host_activity_rail_pointer_layout::HostActivityRailPointerLayout;
use super::host_activity_rail_pointer_target::HostActivityRailPointerTarget;

#[derive(Default)]
pub(crate) struct HostActivityRailPointerBridge {
    pub(super) layout: HostActivityRailPointerLayout,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) targets: BTreeMap<UiNodeId, HostActivityRailPointerTarget>,
}
