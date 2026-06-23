use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use super::host_activity_rail_pointer_layout::HostActivityRailPointerLayout;
use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

#[derive(Default)]
pub(crate) struct HostActivityRailPointerBridge {
    pub(super) layout: HostActivityRailPointerLayout,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) route_intents: EditorRouteIntentMap,
}
