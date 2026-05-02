use std::collections::BTreeMap;

use zircon_runtime::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiFrame,
    tree::{UiInputPolicy, UiTreeNode},
};

use crate::scene::viewport::pointer::{
    candidates::{
        candidate_z_index, interactive_state_flags, passive_state_flags,
        precision_candidates_from_layout,
    },
    constants::{FIRST_CANDIDATE_NODE_ID, ROOT_NODE_ID, VIEWPORT_NODE_ID},
};

use super::ViewportOverlayPointerRouter;

impl ViewportOverlayPointerRouter {
    pub(in crate::scene::viewport::pointer) fn rebuild_surface(&mut self) {
        let viewport_frame = UiFrame::new(
            0.0,
            0.0,
            self.layout.viewport.x.max(1) as f32,
            self.layout.viewport.y.max(1) as f32,
        );
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.viewport.pointer"));
        surface.tree.insert_root(
            UiTreeNode::new(
                ROOT_NODE_ID,
                UiNodePath::new("editor.viewport.pointer.root"),
            )
            .with_frame(viewport_frame)
            .with_state_flags(passive_state_flags())
            .with_input_policy(UiInputPolicy::Receive),
        );
        surface
            .tree
            .insert_child(
                ROOT_NODE_ID,
                UiTreeNode::new(
                    VIEWPORT_NODE_ID,
                    UiNodePath::new("editor.viewport.pointer.viewport"),
                )
                .with_frame(viewport_frame)
                .with_state_flags(interactive_state_flags())
                .with_input_policy(UiInputPolicy::Receive),
            )
            .expect("viewport pointer root must exist");

        let mut candidates = BTreeMap::new();
        let mut next_node_id = FIRST_CANDIDATE_NODE_ID;

        for candidate in precision_candidates_from_layout(&self.layout) {
            let node_id = UiNodeId::new(next_node_id);
            next_node_id += 1;
            let Some(frame) = candidate.shape.hit_frame() else {
                continue;
            };
            surface
                .tree
                .insert_child(
                    VIEWPORT_NODE_ID,
                    UiTreeNode::new(
                        node_id,
                        UiNodePath::new(format!(
                            "editor.viewport.pointer/candidate_{next_node_id}"
                        )),
                    )
                    .with_frame(frame)
                    .with_state_flags(interactive_state_flags())
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_z_index(candidate_z_index(candidate.priority)),
                )
                .expect("viewport pointer viewport must exist");
            candidates.insert(node_id, candidate);
        }

        surface.rebuild();
        self.surface = surface;
        if let Ok(mut shared) = self.shared.lock() {
            shared.candidates = candidates;
            shared.last_route = None;
        }
    }
}
