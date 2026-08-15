use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use crate::ui::retained_host::route_intent::EditorRouteIntentMap;
use crate::ui::retained_host::welcome_recent_geometry::WelcomeRecentLayoutMetrics;

use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;
use super::welcome_recent_pointer_state::WelcomeRecentPointerState;

pub(crate) struct WelcomeRecentPointerBridge {
    pub(in crate::ui::retained_host::welcome_recent_pointer) layout: WelcomeRecentPointerLayout,
    pub(in crate::ui::retained_host::welcome_recent_pointer) state: WelcomeRecentPointerState,
    pub(in crate::ui::retained_host::welcome_recent_pointer) layout_metrics:
        WelcomeRecentLayoutMetrics,
    pub(in crate::ui::retained_host::welcome_recent_pointer) surface: UiSurface,
    pub(in crate::ui::retained_host::welcome_recent_pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::ui::retained_host::welcome_recent_pointer) route_intents: EditorRouteIntentMap,
    #[cfg(test)]
    pub(in crate::ui::retained_host::welcome_recent_pointer) surface_authority_generation: u64,
}

impl WelcomeRecentPointerBridge {
    #[cfg(test)]
    pub(crate) fn surface_node_count_for_test(&self) -> usize {
        self.surface.tree.nodes.len()
    }

    #[cfg(test)]
    pub(crate) const fn surface_authority_generation_for_test(&self) -> u64 {
        self.surface_authority_generation
    }
}
