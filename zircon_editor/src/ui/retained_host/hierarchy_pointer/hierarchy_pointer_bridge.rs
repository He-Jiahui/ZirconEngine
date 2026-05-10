use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::hierarchy_pointer_layout::HierarchyPointerLayout;
use super::hierarchy_pointer_state::HierarchyPointerState;
use super::hierarchy_pointer_target::HierarchyPointerTarget;

#[derive(Default)]
pub(crate) struct HierarchyPointerBridge {
    pub(super) layout: HierarchyPointerLayout,
    pub(super) state: HierarchyPointerState,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) targets: BTreeMap<UiNodeId, HierarchyPointerTarget>,
}
