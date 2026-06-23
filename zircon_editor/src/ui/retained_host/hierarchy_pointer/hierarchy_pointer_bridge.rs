use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

use super::hierarchy_pointer_layout::HierarchyPointerLayout;
use super::hierarchy_pointer_state::HierarchyPointerState;

#[derive(Default)]
pub(crate) struct HierarchyPointerBridge {
    pub(super) layout: HierarchyPointerLayout,
    pub(super) state: HierarchyPointerState,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) route_intents: EditorRouteIntentMap,
}
