use serde::{Deserialize, Serialize};

use crate::ui::tree::{
    UiRuntimeTreeAccessExt, UiRuntimeTreeInteractionExt, UiRuntimeTreeRenderOrderExt,
};
use zircon_runtime_interface::ui::tree::{UiInputPolicy, UiTree};
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiPoint};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiHitTestResult {
    pub top_hit: Option<UiNodeId>,
    pub stacked: Vec<UiNodeId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiHitTestIndex {
    pub draw_order: Vec<UiNodeId>,
}

impl UiHitTestIndex {
    pub fn rebuild(&mut self, tree: &UiTree) {
        self.draw_order = tree.draw_order();
    }

    pub fn hit_test(&self, tree: &UiTree, point: UiPoint) -> UiHitTestResult {
        let mut stacked = Vec::new();

        for node_id in self.draw_order.iter().rev().copied() {
            let Some(node) = tree.node(node_id) else {
                continue;
            };
            if !tree.is_visible_in_tree(node_id).unwrap_or(false) {
                continue;
            }
            if !tree.supports_pointer(node_id).unwrap_or(false) {
                continue;
            }
            if tree
                .effective_input_policy(node_id)
                .is_ok_and(|policy| policy == UiInputPolicy::Ignore)
            {
                continue;
            }
            if !node.layout_cache.frame.contains_point(point) {
                continue;
            }
            if !tree.passes_clip_chain(node_id, point).unwrap_or(false) {
                continue;
            }
            stacked.push(node_id);
        }

        UiHitTestResult {
            top_hit: stacked.first().copied(),
            stacked,
        }
    }
}
