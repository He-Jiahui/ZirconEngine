use std::collections::BTreeMap;

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher, event_ui::UiNodeId, layout::UiFrame, surface::UiSurface,
};

use super::host_page_pointer_layout::HostPagePointerLayout;
use super::host_page_pointer_target::HostPagePointerTarget;

#[derive(Default)]
pub(crate) struct HostPagePointerBridge {
    pub(super) layout: HostPagePointerLayout,
    pub(super) measured_frames: Vec<Option<UiFrame>>,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) targets: BTreeMap<UiNodeId, HostPagePointerTarget>,
}
