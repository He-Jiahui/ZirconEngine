use std::collections::HashMap;

use zircon_runtime_interface::ui::{
    dispatch::{UiComponentEventReport, UiPointerDispatchResult},
    event_ui::{UiNodeId, UiRouteId},
};

use crate::core::editing::intent::EditorIntent;
use crate::ui::retained_host::{
    activity_rail_pointer::HostActivityRailPointerRoute, menu_pointer::HostMenuPointerRouteIntent,
    shell_pointer::HostShellPointerRoute, viewport_toolbar_pointer::ViewportToolbarPointerRoute,
};

#[derive(Clone, Debug)]
pub(crate) enum EditorRouteIntent {
    Editor(EditorIntent),
    ShellPointer(HostShellPointerRoute),
    Menu(HostMenuPointerRouteIntent),
    ActivityRail(HostActivityRailPointerRoute),
    ViewportToolbar(ViewportToolbarPointerRoute),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EditorRouteIntentMap {
    bindings_by_node: HashMap<UiNodeId, EditorRouteBinding>,
}

#[derive(Clone, Debug)]
struct EditorRouteBinding {
    route_id: UiRouteId,
    intent: EditorRouteIntent,
}

impl EditorRouteIntentMap {
    pub(crate) fn bind_node(
        &mut self,
        node_id: UiNodeId,
        route_id: UiRouteId,
        intent: EditorRouteIntent,
    ) {
        self.bindings_by_node
            .insert(node_id, EditorRouteBinding { route_id, intent });
    }

    pub(crate) fn route_id_for_node(&self, node_id: UiNodeId) -> Option<UiRouteId> {
        self.bindings_by_node
            .get(&node_id)
            .map(|binding| binding.route_id)
    }

    pub(crate) fn intent_for_node(&self, node_id: UiNodeId) -> Option<&EditorRouteIntent> {
        self.bindings_by_node
            .get(&node_id)
            .map(|binding| &binding.intent)
    }

    pub(crate) fn intent_for(&self, event: &UiComponentEventReport) -> Option<&EditorRouteIntent> {
        self.intent_for_node(event.target)
    }

    pub(crate) fn intent_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<&EditorRouteIntent> {
        pointer_dispatch_route_node(dispatch).and_then(|node_id| self.intent_for_node(node_id))
    }

    pub(crate) fn shell_pointer_route_for_node(
        &self,
        node_id: UiNodeId,
    ) -> Option<HostShellPointerRoute> {
        match self.intent_for_node(node_id)? {
            EditorRouteIntent::ShellPointer(route) => Some(route.clone()),
            _ => None,
        }
    }

    pub(crate) fn menu_route_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<HostMenuPointerRouteIntent> {
        match self.intent_for_pointer_dispatch(dispatch)? {
            EditorRouteIntent::Menu(route) => Some(route.clone()),
            _ => None,
        }
    }

    pub(crate) fn activity_rail_route_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<HostActivityRailPointerRoute> {
        match self.intent_for_pointer_dispatch(dispatch)? {
            EditorRouteIntent::ActivityRail(route) => Some(*route),
            _ => None,
        }
    }

    pub(crate) fn viewport_toolbar_route_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<ViewportToolbarPointerRoute> {
        match self.intent_for_pointer_dispatch(dispatch)? {
            EditorRouteIntent::ViewportToolbar(route) => Some(route.clone()),
            _ => None,
        }
    }
}

fn pointer_dispatch_route_node(dispatch: &UiPointerDispatchResult) -> Option<UiNodeId> {
    dispatch.handled_by.or(dispatch.route.target)
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn route_intent_map_uses_one_hash_index_for_hot_pointer_lookup() {
        let source = include_str!("map.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");
        assert!(implementation.contains("HashMap<UiNodeId, EditorRouteBinding>"));
        assert_eq!(implementation.matches("HashMap<").count(), 1);
        assert!(!implementation.contains("BTreeMap"));
    }
}
