use std::collections::BTreeSet;

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher,
    surface::{UiAuthoredGeometryPublication, UiSurface},
};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiRouteId, UiTreeId},
    layout::{UiFrame, UiSize},
    tree::{UiInputPolicy, UiTreeNode},
};

use crate::ui::retained_host::route_intent::{EditorRouteIntent, EditorRouteIntentMap};

use super::base_state::base_state;
use super::constants::{
    CONTROL_NODE_ID_BASE, ROOT_NODE_ID, SURFACE_NODE_ID_BASE, VIEWPORT_TOOLBAR_ROUTE_ID_BASE,
    VIEWPORT_TOOLBAR_SURFACE_STRIDE,
};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::root_frame::root_frame;
use super::route_for_control::route_for_control;
use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;

impl ViewportToolbarPointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        self.apply_surface_delta(ViewportToolbarSurfaceDelta::Topology);
    }

    pub(super) fn apply_surface_delta(&mut self, delta: ViewportToolbarSurfaceDelta) {
        match delta {
            ViewportToolbarSurfaceDelta::NoChange => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "viewport_toolbar.pointer.surface_delta_no_change_count",
                    1_u8,
                );
            }
            ViewportToolbarSurfaceDelta::Geometry(changes) => {
                self.apply_geometry_delta(changes);
            }
            ViewportToolbarSurfaceDelta::Topology => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "viewport_toolbar.pointer.surface_delta_topology_count",
                    1_u8,
                );
                self.rebuild_surface_from_scratch();
            }
        }
    }

    fn apply_geometry_delta(&mut self, changes: Vec<ViewportToolbarNodeFrameChange>) {
        if changes.is_empty() {
            self.apply_surface_delta(ViewportToolbarSurfaceDelta::NoChange);
            return;
        }
        let root_frame = root_frame(&self.layout);
        let observed_topology_generation = self.surface.tree.layout_order_generation();
        let mut changed_node_ids = BTreeSet::new();
        for change in changes {
            patch_retained_node_frame(
                &mut self.surface,
                change.node_id,
                change.frame,
                &mut changed_node_ids,
            );
        }
        if changed_node_ids.is_empty() {
            self.apply_surface_delta(ViewportToolbarSurfaceDelta::NoChange);
            return;
        }
        zircon_runtime::profile_counter!(
            "editor",
            "viewport_toolbar.pointer.surface_delta_geometry_count",
            1_u8,
        );
        zircon_runtime::profile_counter!(
            "editor",
            "viewport_toolbar.pointer.surface_geometry_patch_node_count",
            changed_node_ids.len(),
        );
        match self.surface.publish_authored_geometry(
            UiSize::new(root_frame.width, root_frame.height),
            &changed_node_ids,
            observed_topology_generation,
        ) {
            UiAuthoredGeometryPublication::Local(_) => zircon_runtime::profile_counter!(
                "editor",
                "viewport_toolbar.pointer.surface_geometry_local_publication_count",
                1_u8,
            ),
            UiAuthoredGeometryPublication::FullFallback { .. } => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "viewport_toolbar.pointer.surface_geometry_fallback_count",
                    1_u8,
                );
            }
            UiAuthoredGeometryPublication::Unchanged => {}
        }
    }

    fn rebuild_surface_from_scratch(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.viewport_toolbar.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut route_intents = EditorRouteIntentMap::default();
        let root_frame = root_frame(&self.layout);

        surface.tree.insert_root(
            UiTreeNode::new(
                ROOT_NODE_ID,
                UiNodePath::new("editor.viewport_toolbar.root"),
            )
            .with_frame(root_frame)
            .with_state_flags(base_state(false)),
        );

        for (surface_index, surface_layout) in self.layout.surfaces.iter().enumerate() {
            let surface_node_id = UiNodeId::new(SURFACE_NODE_ID_BASE + surface_index as u64);
            surface
                .tree
                .insert_child(
                    ROOT_NODE_ID,
                    UiTreeNode::new(
                        surface_node_id,
                        UiNodePath::new(format!("editor.viewport_toolbar/{}", surface_layout.key)),
                    )
                    .with_frame(surface_layout.frame)
                    .with_z_index(10 + surface_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("viewport toolbar root must exist");

            let Some(controls) = self.controls_by_surface.get(surface_layout.key.as_str()) else {
                continue;
            };
            for (control_index, control) in controls.iter().enumerate() {
                let route = route_for_control(surface_layout.key.as_str(), &control.action_key)
                    .expect("viewport toolbar action must stay valid");
                let control_node_id =
                    viewport_toolbar_control_node_id(surface_index, control_index);
                surface
                    .tree
                    .insert_child(
                        surface_node_id,
                        UiTreeNode::new(
                            control_node_id,
                            UiNodePath::new(format!(
                                "editor.viewport_toolbar/{}/{}",
                                surface_layout.key, control.action_key
                            )),
                        )
                        .with_frame(control.frame)
                        .with_z_index(100 + control_index as i32)
                        .with_input_policy(UiInputPolicy::Receive)
                        .with_state_flags(base_state(true)),
                    )
                    .expect("viewport toolbar surface must exist");
                register_handled_pointer_node(&mut dispatcher, control_node_id);
                route_intents.bind_node(
                    control_node_id,
                    viewport_toolbar_route_id(surface_index, control_index),
                    EditorRouteIntent::ViewportToolbar(route),
                );
            }
        }

        surface.rebuild_authored_frames(UiSize::new(root_frame.width, root_frame.height));
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.route_intents = route_intents;
    }
}

fn patch_retained_node_frame(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    frame: UiFrame,
    changed_node_ids: &mut BTreeSet<UiNodeId>,
) {
    if surface
        .tree
        .node(node_id)
        .is_some_and(|node| node.layout_cache.frame == frame)
    {
        return;
    }
    surface
        .tree
        .node_mut(node_id)
        .expect("toolbar geometry receipt must reference the retained topology")
        .layout_cache
        .frame = frame;
    changed_node_ids.insert(node_id);
}

pub(super) const fn viewport_toolbar_surface_node_id(surface_index: usize) -> UiNodeId {
    UiNodeId::new(SURFACE_NODE_ID_BASE + surface_index as u64)
}

pub(super) const fn viewport_toolbar_control_node_id(
    surface_index: usize,
    control_index: usize,
) -> UiNodeId {
    UiNodeId::new(
        CONTROL_NODE_ID_BASE
            + surface_index as u64 * VIEWPORT_TOOLBAR_SURFACE_STRIDE
            + control_index as u64,
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ViewportToolbarNodeFrameChange {
    pub(super) node_id: UiNodeId,
    pub(super) frame: UiFrame,
}

#[derive(Debug, PartialEq)]
pub(super) enum ViewportToolbarSurfaceDelta {
    NoChange,
    Geometry(Vec<ViewportToolbarNodeFrameChange>),
    Topology,
}

const fn viewport_toolbar_route_id(surface_index: usize, control_index: usize) -> UiRouteId {
    UiRouteId::new(
        VIEWPORT_TOOLBAR_ROUTE_ID_BASE
            + surface_index as u64 * VIEWPORT_TOOLBAR_SURFACE_STRIDE
            + control_index as u64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::viewport_toolbar_pointer::{
        viewport_toolbar_pointer_control::ViewportToolbarPointerControl,
        viewport_toolbar_pointer_surface::ViewportToolbarPointerSurface,
    };

    #[test]
    fn stable_toolbar_control_frame_uses_exact_runtime_geometry_publication() {
        let mut bridge = ViewportToolbarPointerBridge::new();
        bridge.layout.surfaces = vec![ViewportToolbarPointerSurface {
            key: "scene.main".to_string(),
            frame: UiFrame::new(0.0, 0.0, 320.0, 40.0),
        }];
        bridge.controls_by_surface.insert(
            "scene.main".to_string(),
            vec![ViewportToolbarPointerControl {
                action_key: "mode.select".to_string(),
                frame: UiFrame::new(8.0, 8.0, 24.0, 24.0),
            }],
        );
        bridge.rebuild_surface_from_scratch();

        bridge
            .controls_by_surface
            .get_mut("scene.main")
            .expect("toolbar controls must exist")[0]
            .frame = UiFrame::new(40.0, 8.0, 24.0, 24.0);
        bridge.apply_surface_delta(ViewportToolbarSurfaceDelta::Geometry(vec![
            ViewportToolbarNodeFrameChange {
                node_id: viewport_toolbar_control_node_id(0, 0),
                frame: UiFrame::new(40.0, 8.0, 24.0, 24.0),
            },
        ]));

        assert_eq!(
            bridge
                .surface
                .last_rebuild_report
                .arranged_outer_node_visit_count,
            1
        );
        assert_eq!(
            bridge
                .surface
                .last_rebuild_report
                .hit_grid_outer_node_visit_count,
            1
        );
        assert_eq!(
            bridge
                .surface
                .last_rebuild_report
                .render_outer_node_visit_count,
            1
        );
    }
}
