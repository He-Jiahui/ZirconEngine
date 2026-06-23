use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::host_menu_pointer_state::HostMenuPointerState;
use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

pub(crate) struct HostMenuPointerBridge {
    pub(in crate::ui::retained_host::menu_pointer) layout: HostMenuPointerLayout,
    pub(in crate::ui::retained_host::menu_pointer) state: HostMenuPointerState,
    pub(in crate::ui::retained_host::menu_pointer) surface: UiSurface,
    pub(in crate::ui::retained_host::menu_pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::ui::retained_host::menu_pointer) route_intents: EditorRouteIntentMap,
}
