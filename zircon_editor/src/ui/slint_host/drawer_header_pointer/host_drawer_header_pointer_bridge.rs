use std::collections::BTreeMap;

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher, event_ui::UiNodeId, layout::UiFrame, surface::UiSurface,
};

use super::host_drawer_header_pointer_layout::HostDrawerHeaderPointerLayout;
use super::host_drawer_header_pointer_target::HostDrawerHeaderPointerTarget;

#[derive(Default)]
pub(crate) struct HostDrawerHeaderPointerBridge {
    pub(super) layout: HostDrawerHeaderPointerLayout,
    pub(super) measured_frames: BTreeMap<String, Vec<Option<UiFrame>>>,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) targets: BTreeMap<UiNodeId, HostDrawerHeaderPointerTarget>,
}
