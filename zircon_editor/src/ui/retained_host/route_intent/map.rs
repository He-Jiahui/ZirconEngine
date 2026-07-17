use std::collections::HashMap;

use zircon_runtime_interface::ui::{
    dispatch::{UiComponentEventReport, UiPointerDispatchResult},
    event_ui::{UiNodeId, UiRouteId},
};

use crate::core::editing::intent::EditorIntent;
use crate::ui::retained_host::{
    activity_rail_pointer::HostActivityRailPointerRoute, detail_pointer::ScrollSurfacePointerRoute,
    document_tab_pointer::HostDocumentTabPointerRoute,
    drawer_header_pointer::HostDrawerHeaderPointerRoute, hierarchy_pointer::HierarchyPointerRoute,
    host_page_pointer::HostPagePointerRoute, menu_pointer::HostMenuPointerRouteIntent,
    shell_pointer::HostShellPointerRoute, viewport_toolbar_pointer::ViewportToolbarPointerRoute,
    welcome_recent_pointer::WelcomeRecentPointerRouteIntent,
};

#[derive(Clone, Debug)]
pub(crate) enum EditorRouteIntent {
    Editor(EditorIntent),
    ShellPointer(HostShellPointerRoute),
    DocumentTab(HostDocumentTabPointerRoute),
    DrawerHeader(HostDrawerHeaderPointerRoute),
    Menu(HostMenuPointerRouteIntent),
    ActivityRail(HostActivityRailPointerRoute),
    Hierarchy(HierarchyPointerRoute),
    Detail(ScrollSurfacePointerRoute),
    HostPage(HostPagePointerRoute),
    ViewportToolbar(ViewportToolbarPointerRoute),
    WelcomeRecent(WelcomeRecentPointerRouteIntent),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EditorRouteIntentMap {
    route_by_node: HashMap<UiNodeId, UiRouteId>,
    intent_by_route: HashMap<UiRouteId, EditorRouteIntent>,
}

impl EditorRouteIntentMap {
    pub(crate) fn bind_node(
        &mut self,
        node_id: UiNodeId,
        route_id: UiRouteId,
        intent: EditorRouteIntent,
    ) {
        self.route_by_node.insert(node_id, route_id);
        self.intent_by_route.insert(route_id, intent);
    }

    pub(crate) fn route_id_for_node(&self, node_id: UiNodeId) -> Option<UiRouteId> {
        self.route_by_node.get(&node_id).copied()
    }

    pub(crate) fn intent_for_route_id(&self, route_id: UiRouteId) -> Option<&EditorRouteIntent> {
        self.intent_by_route.get(&route_id)
    }

    pub(crate) fn intent_for_node(&self, node_id: UiNodeId) -> Option<&EditorRouteIntent> {
        self.route_id_for_node(node_id)
            .and_then(|route_id| self.intent_for_route_id(route_id))
    }

    pub(crate) fn intent_for(&self, event: &UiComponentEventReport) -> Option<&EditorRouteIntent> {
        self.intent_for_node(event.target)
    }

    pub(crate) fn route_id_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<UiRouteId> {
        pointer_dispatch_route_node(dispatch).and_then(|node_id| self.route_id_for_node(node_id))
    }

    pub(crate) fn intent_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<&EditorRouteIntent> {
        self.route_id_for_pointer_dispatch(dispatch)
            .and_then(|route_id| self.intent_for_route_id(route_id))
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

    pub(crate) fn document_tab_route_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<HostDocumentTabPointerRoute> {
        match self.intent_for_pointer_dispatch(dispatch)? {
            EditorRouteIntent::DocumentTab(route) => Some(route.clone()),
            _ => None,
        }
    }

    pub(crate) fn drawer_header_route_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<HostDrawerHeaderPointerRoute> {
        match self.intent_for_pointer_dispatch(dispatch)? {
            EditorRouteIntent::DrawerHeader(route) => Some(route.clone()),
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
            EditorRouteIntent::ActivityRail(route) => Some(route.clone()),
            _ => None,
        }
    }

    pub(crate) fn hierarchy_route_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<HierarchyPointerRoute> {
        match self.intent_for_pointer_dispatch(dispatch)? {
            EditorRouteIntent::Hierarchy(route) => Some(route.clone()),
            _ => None,
        }
    }

    pub(crate) fn detail_route_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<ScrollSurfacePointerRoute> {
        match self.intent_for_pointer_dispatch(dispatch)? {
            EditorRouteIntent::Detail(route) => Some(*route),
            _ => None,
        }
    }

    pub(crate) fn host_page_route_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<HostPagePointerRoute> {
        match self.intent_for_pointer_dispatch(dispatch)? {
            EditorRouteIntent::HostPage(route) => Some(route.clone()),
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

    pub(crate) fn welcome_recent_route_for_pointer_dispatch(
        &self,
        dispatch: &UiPointerDispatchResult,
    ) -> Option<WelcomeRecentPointerRouteIntent> {
        match self.intent_for_pointer_dispatch(dispatch)? {
            EditorRouteIntent::WelcomeRecent(route) => Some(route.clone()),
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
    fn route_intent_map_uses_hash_indices_for_hot_pointer_lookup() {
        let source = include_str!("map.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");
        assert!(implementation.contains("HashMap<UiNodeId, UiRouteId>"));
        assert!(implementation.contains("HashMap<UiRouteId, EditorRouteIntent>"));
        assert!(!implementation.contains("BTreeMap"));
    }
}
