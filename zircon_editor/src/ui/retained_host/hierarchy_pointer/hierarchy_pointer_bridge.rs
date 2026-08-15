use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

use super::hierarchy_pointer_layout::HierarchyPointerLayout;
use super::hierarchy_pointer_state::HierarchyPointerState;
use super::row_metrics::HierarchyRowMetrics;

#[derive(Default)]
pub(crate) struct HierarchyPointerBridge {
    pub(super) layout: HierarchyPointerLayout,
    pub(super) state: HierarchyPointerState,
    pub(super) row_metrics: HierarchyRowMetrics,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) route_intents: EditorRouteIntentMap,
    #[cfg(test)]
    pub(super) surface_authority_generation: u64,
}

impl HierarchyPointerBridge {
    #[cfg(test)]
    pub(crate) fn surface_node_count_for_test(&self) -> usize {
        self.surface.tree.nodes.len()
    }

    #[cfg(test)]
    pub(crate) const fn surface_authority_generation_for_test(&self) -> u64 {
        self.surface_authority_generation
    }
}
