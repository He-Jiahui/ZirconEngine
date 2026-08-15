use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::layout::UiFrame;

use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

use super::host_page_pointer_layout::HostPagePointerLayout;

#[derive(Default)]
pub(crate) struct HostPagePointerBridge {
    pub(super) layout: HostPagePointerLayout,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) route_intents: EditorRouteIntentMap,
    pub(super) measured_frames: Vec<Option<(UiFrame, Option<UiFrame>)>>,
    pub(super) tab_positions_by_item: Vec<Option<usize>>,
    #[cfg(test)]
    pub(super) surface_authority_generation: u64,
}

impl HostPagePointerBridge {
    #[cfg(test)]
    pub(crate) const fn debug_surface_authority_generation(&self) -> u64 {
        self.surface_authority_generation
    }
}
