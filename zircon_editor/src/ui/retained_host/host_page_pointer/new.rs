use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::{event_ui::UiTreeId, layout::UiFrame};

use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_layout::HostPagePointerLayout;

impl HostPagePointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: HostPagePointerLayout {
                strip_frame: UiFrame::default(),
                items: Vec::new(),
                tabs: Vec::new(),
                overflow: None,
            },
            surface: UiSurface::new(UiTreeId::new("zircon.editor.host_page.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            route_intents: EditorRouteIntentMap::default(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
