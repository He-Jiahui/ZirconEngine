use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    layout::UiFrame,
    tree::{UiInputPolicy, UiTreeNode},
};

use crate::ui::retained_host::route_intent::{EditorRouteIntent, EditorRouteIntentMap};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

use super::constants::{ROOT_NODE_ID, VIEWPORT_NODE_ID};
use super::helper::{base_state, list_surface_route_id, viewport_frame};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_route_intent::WelcomeRecentPointerRouteIntent;

impl WelcomeRecentPointerBridge {
    pub(in crate::ui::retained_host::welcome_recent_pointer) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.welcome.recent_pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut route_intents = EditorRouteIntentMap::default();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.welcome.recent.root"))
                .with_frame(UiFrame::new(
                    0.0,
                    0.0,
                    self.layout.pane_size.width.max(0.0),
                    self.layout.pane_size.height.max(0.0),
                ))
                .with_state_flags(base_state(false)),
        );

        let viewport = viewport_frame(&self.layout, self.layout_metrics);
        surface
            .tree
            .insert_child(
                ROOT_NODE_ID,
                UiTreeNode::new(
                    VIEWPORT_NODE_ID,
                    UiNodePath::new("editor.welcome.recent.viewport"),
                )
                .with_frame(viewport)
                .with_z_index(10)
                .with_input_policy(UiInputPolicy::Receive)
                .with_clip_to_bounds(true)
                .with_state_flags(base_state(true)),
            )
            .expect("welcome recent pointer root must exist");
        register_handled_pointer_node(&mut dispatcher, VIEWPORT_NODE_ID);
        route_intents.bind_node(
            VIEWPORT_NODE_ID,
            list_surface_route_id(),
            EditorRouteIntent::WelcomeRecent(WelcomeRecentPointerRouteIntent::ListSurface),
        );

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.route_intents = route_intents;
        #[cfg(test)]
        {
            self.surface_authority_generation = self.surface_authority_generation.saturating_add(1);
        }
        record_current_ui_perf_counter(UiPerfCounter::WelcomeRecentSurfaceRebuildCount, 1.0);
        record_current_ui_perf_counter(UiPerfCounter::WelcomeRecentAuthorityRebuildCount, 1.0);
        record_current_ui_perf_counter(UiPerfCounter::WelcomeRecentRowInsertCount, 0.0);
        record_current_ui_perf_counter(UiPerfCounter::WelcomeRecentDispatcherRebuildCount, 1.0);
        record_current_ui_perf_counter(UiPerfCounter::WelcomeRecentRouteMapRebuildCount, 1.0);
    }

    pub(in crate::ui::retained_host::welcome_recent_pointer) fn patch_surface_geometry(&mut self) {
        let root_frame = UiFrame::new(
            0.0,
            0.0,
            self.layout.pane_size.width.max(0.0),
            self.layout.pane_size.height.max(0.0),
        );
        let viewport = viewport_frame(&self.layout, self.layout_metrics);
        let Some(root) = self.surface.tree.node(ROOT_NODE_ID) else {
            self.rebuild_surface();
            return;
        };
        let Some(current_viewport) = self.surface.tree.node(VIEWPORT_NODE_ID) else {
            self.rebuild_surface();
            return;
        };
        let root_changed = root.layout_cache.frame != root_frame;
        let viewport_changed = current_viewport.layout_cache.frame != viewport;
        if !root_changed && !viewport_changed {
            return;
        }
        if root_changed {
            self.surface
                .tree
                .node_mut(ROOT_NODE_ID)
                .expect("validated welcome recent root must exist")
                .layout_cache
                .frame = root_frame;
        }
        if viewport_changed {
            self.surface
                .tree
                .node_mut(VIEWPORT_NODE_ID)
                .expect("validated welcome recent viewport must exist")
                .layout_cache
                .frame = viewport;
        }
        self.surface.rebuild();
        record_current_ui_perf_counter(UiPerfCounter::WelcomeRecentSurfaceRebuildCount, 1.0);
        record_current_ui_perf_counter(UiPerfCounter::WelcomeRecentGeometryPatchCount, 1.0);
    }
}
