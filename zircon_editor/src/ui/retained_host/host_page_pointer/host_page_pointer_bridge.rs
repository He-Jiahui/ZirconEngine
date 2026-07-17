use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

use super::host_page_pointer_layout::HostPagePointerLayout;

#[derive(Default)]
pub(crate) struct HostPagePointerBridge {
    pub(super) layout: HostPagePointerLayout,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) route_intents: EditorRouteIntentMap,
}
