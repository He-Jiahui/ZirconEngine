use std::collections::HashMap;
use std::sync::Arc;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::host_menu_pointer_state::HostMenuPointerState;
use super::menu_item_spec::MenuItemSpec;
use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

pub(crate) struct HostMenuPointerBridge {
    pub(in crate::ui::retained_host::menu_pointer) layout: Arc<HostMenuPointerLayout>,
    pub(in crate::ui::retained_host::menu_pointer) state: HostMenuPointerState,
    pub(in crate::ui::retained_host::menu_pointer) surface: UiSurface,
    pub(in crate::ui::retained_host::menu_pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::ui::retained_host::menu_pointer) route_intents: EditorRouteIntentMap,
    pub(in crate::ui::retained_host::menu_pointer) popup_menu_index: Option<usize>,
    pub(in crate::ui::retained_host::menu_pointer) popup_items: Vec<MenuItemSpec>,
    pub(in crate::ui::retained_host::menu_pointer) popup_route_indices: HashMap<Vec<usize>, usize>,
    #[cfg(test)]
    pub(in crate::ui::retained_host::menu_pointer) surface_authority_generation: u64,
}

impl HostMenuPointerBridge {
    #[cfg(test)]
    pub(crate) fn surface_node_count_for_test(&self) -> usize {
        self.surface.tree.nodes.len()
    }

    #[cfg(test)]
    pub(crate) const fn surface_authority_generation_for_test(&self) -> u64 {
        self.surface_authority_generation
    }
}
