use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EditorRouteIntentHandle {
    generation: u64,
    pub(crate) node_id: UiNodeId,
    pub(crate) route_id: UiRouteId,
}

#[derive(Debug)]
pub(crate) struct EditorRouteIntentMap {
    generation: u64,
    bindings_by_node: HashMap<UiNodeId, EditorRouteBinding>,
}

static NEXT_ROUTE_INTENT_GENERATION: AtomicU64 = AtomicU64::new(1);

fn next_route_intent_generation() -> u64 {
    NEXT_ROUTE_INTENT_GENERATION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |generation| {
            generation.checked_add(1)
        })
        .expect("route intent generation exhausted")
}

impl Default for EditorRouteIntentMap {
    fn default() -> Self {
        Self {
            generation: next_route_intent_generation(),
            bindings_by_node: HashMap::new(),
        }
    }
}

impl Clone for EditorRouteIntentMap {
    fn clone(&self) -> Self {
        Self {
            generation: next_route_intent_generation(),
            bindings_by_node: self.bindings_by_node.clone(),
        }
    }
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
        self.generation = next_route_intent_generation();
        self.bindings_by_node
            .insert(node_id, EditorRouteBinding { route_id, intent });
    }

    pub(crate) fn handle_for_node(&self, node_id: UiNodeId) -> Option<EditorRouteIntentHandle> {
        let binding = self.bindings_by_node.get(&node_id)?;
        Some(EditorRouteIntentHandle {
            generation: self.generation,
            node_id,
            route_id: binding.route_id,
        })
    }

    pub(crate) fn handle_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<EditorRouteIntentHandle> {
        pointer_dispatch_route_node(dispatch).and_then(|node_id| self.handle_for_node(node_id))
    }

    pub(crate) fn resolve_handle(
        &self,
        handle: EditorRouteIntentHandle,
    ) -> Option<&EditorRouteIntent> {
        if handle.generation != self.generation {
            return None;
        }
        let binding = self.bindings_by_node.get(&handle.node_id)?;
        (binding.route_id == handle.route_id).then_some(&binding.intent)
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
#[path = "tests/map_performance_tests.rs"]
mod performance_tests;
