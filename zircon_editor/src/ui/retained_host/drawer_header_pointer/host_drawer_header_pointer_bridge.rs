use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::layout::UiFrame;

use super::host_drawer_header_pointer_layout::HostDrawerHeaderPointerLayout;
use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

#[derive(Default)]
pub(crate) struct HostDrawerHeaderPointerBridge {
    pub(super) layout: HostDrawerHeaderPointerLayout,
    pub(super) measured_frames: BTreeMap<String, Vec<Option<UiFrame>>>,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) route_intents: EditorRouteIntentMap,
}
