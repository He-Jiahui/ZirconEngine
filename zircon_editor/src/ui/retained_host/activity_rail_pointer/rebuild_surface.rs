use std::collections::BTreeSet;

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher,
    surface::{UiAuthoredGeometryPublication, UiSurface},
};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{UiFrame, UiSize},
    tree::UiTreeNode,
};

use super::base_state::base_state;
use super::constants::{LEFT_STRIP_NODE_ID, RIGHT_STRIP_NODE_ID, ROOT_NODE_ID};
use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::insert_strip::insert_strip;
use super::root_frame::root_frame;
use super::surface_delta::ActivityRailNodeFrameChange;
use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

impl HostActivityRailPointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        self.rebuild_surface_from_scratch();
    }

    pub(super) fn apply_geometry_delta(&mut self, changes: Vec<ActivityRailNodeFrameChange>) {
        if changes.is_empty() {
            return;
        }
        if changes
            .iter()
            .any(|change| self.surface.tree.node(change.node_id).is_none())
        {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.activity_rail.surface_geometry_authority_miss_count",
                1
            );
            self.rebuild_surface_from_scratch();
            return;
        }
        let root_frame = root_frame(&self.layout);
        let observed_topology_generation = self.surface.tree.layout_order_generation();
        let mut changed_node_ids = BTreeSet::new();
        for change in changes {
            if patch_retained_node_frame(&mut self.surface, change.node_id, change.frame) {
                changed_node_ids.insert(change.node_id);
            }
        }
        if changed_node_ids.is_empty() {
            return;
        }
        zircon_runtime::profile_counter!(
            "editor",
            "ui.activity_rail.surface_geometry_patch_node_count",
            changed_node_ids.len()
        );
        match self.surface.publish_authored_geometry(
            UiSize::new(root_frame.width, root_frame.height),
            &changed_node_ids,
            observed_topology_generation,
        ) {
            UiAuthoredGeometryPublication::Local(_) => zircon_runtime::profile_counter!(
                "editor",
                "ui.activity_rail.surface_geometry_local_publication_count",
                1
            ),
            UiAuthoredGeometryPublication::FullFallback { .. } => zircon_runtime::profile_counter!(
                "editor",
                "ui.activity_rail.surface_geometry_fallback_count",
                1
            ),
            UiAuthoredGeometryPublication::Unchanged => {}
        }
    }

    fn rebuild_surface_from_scratch(&mut self) {
        zircon_runtime::profile_counter!("editor", "ui.activity_rail.surface_rebuild_count", 1);
        zircon_runtime::profile_counter!(
            "editor",
            "ui.activity_rail.surface_rebuild_button_count",
            self.layout.left_tabs.len() + self.layout.right_tabs.len()
        );
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.activity_rail.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut route_intents = EditorRouteIntentMap::default();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.activity_rail.root"))
                .with_frame(root_frame(&self.layout))
                .with_state_flags(base_state(false)),
        );

        insert_strip(
            &mut surface,
            &mut dispatcher,
            &mut route_intents,
            ROOT_NODE_ID,
            LEFT_STRIP_NODE_ID,
            "editor.activity_rail.left",
            self.layout.left_strip_frame,
            &self.layout.left_tabs,
            super::host_activity_rail_pointer_side::HostActivityRailPointerSide::Left,
        );
        insert_strip(
            &mut surface,
            &mut dispatcher,
            &mut route_intents,
            ROOT_NODE_ID,
            RIGHT_STRIP_NODE_ID,
            "editor.activity_rail.right",
            self.layout.right_strip_frame,
            &self.layout.right_tabs,
            super::host_activity_rail_pointer_side::HostActivityRailPointerSide::Right,
        );
        surface.rebuild_authored_frames(UiSize::new(
            root_frame(&self.layout).width,
            root_frame(&self.layout).height,
        ));

        self.surface = surface;
        self.dispatcher = dispatcher;
        self.route_intents = route_intents;
        #[cfg(test)]
        {
            self.surface_authority_generation = self.surface_authority_generation.saturating_add(1);
        }
    }
}

fn patch_retained_node_frame(surface: &mut UiSurface, node_id: UiNodeId, frame: UiFrame) -> bool {
    let Some(node) = surface.tree.node_mut(node_id) else {
        return false;
    };
    if node.layout_cache.frame == frame {
        return false;
    }
    node.layout_cache.frame = frame;
    true
}
