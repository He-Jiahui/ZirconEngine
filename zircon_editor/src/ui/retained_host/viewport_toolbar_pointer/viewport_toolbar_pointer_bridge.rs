use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::active_viewport_toolbar_control::ActiveViewportToolbarControl;
use super::viewport_toolbar_pointer_layout::ViewportToolbarPointerLayout;
use super::viewport_toolbar_pointer_target::ViewportToolbarPointerTarget;

#[derive(Default)]
pub(crate) struct ViewportToolbarPointerBridge {
    pub(super) layout: ViewportToolbarPointerLayout,
    pub(super) active_controls: BTreeMap<String, ActiveViewportToolbarControl>,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) targets: BTreeMap<UiNodeId, ViewportToolbarPointerTarget>,
}
