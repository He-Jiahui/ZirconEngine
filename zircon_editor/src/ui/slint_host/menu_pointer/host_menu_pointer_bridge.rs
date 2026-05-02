use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::host_menu_pointer_state::HostMenuPointerState;
use super::host_menu_pointer_target::HostMenuPointerTarget;

pub(crate) struct HostMenuPointerBridge {
    pub(in crate::ui::slint_host::menu_pointer) layout: HostMenuPointerLayout,
    pub(in crate::ui::slint_host::menu_pointer) state: HostMenuPointerState,
    pub(in crate::ui::slint_host::menu_pointer) surface: UiSurface,
    pub(in crate::ui::slint_host::menu_pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::ui::slint_host::menu_pointer) targets: BTreeMap<UiNodeId, HostMenuPointerTarget>,
}
