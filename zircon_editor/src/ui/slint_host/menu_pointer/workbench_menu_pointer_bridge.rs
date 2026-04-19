use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, event_ui::UiNodeId, surface::UiSurface};

use super::workbench_menu_pointer_layout::WorkbenchMenuPointerLayout;
use super::workbench_menu_pointer_state::WorkbenchMenuPointerState;
use super::workbench_menu_pointer_target::WorkbenchMenuPointerTarget;

pub(crate) struct WorkbenchMenuPointerBridge {
    pub(in crate::ui::slint_host::menu_pointer) layout: WorkbenchMenuPointerLayout,
    pub(in crate::ui::slint_host::menu_pointer) state: WorkbenchMenuPointerState,
    pub(in crate::ui::slint_host::menu_pointer) surface: UiSurface,
    pub(in crate::ui::slint_host::menu_pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::ui::slint_host::menu_pointer) targets:
        BTreeMap<UiNodeId, WorkbenchMenuPointerTarget>,
}
