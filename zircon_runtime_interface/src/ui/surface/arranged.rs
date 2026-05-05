use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use crate::ui::layout::UiFrame;
use crate::ui::tree::{UiInputPolicy, UiVisibility};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiArrangedNode {
    pub node_id: UiNodeId,
    pub node_path: UiNodePath,
    pub parent: Option<UiNodeId>,
    pub children: Vec<UiNodeId>,
    pub frame: UiFrame,
    pub clip_frame: UiFrame,
    pub z_index: i32,
    pub paint_order: u64,
    pub visibility: UiVisibility,
    pub input_policy: UiInputPolicy,
    pub enabled: bool,
    pub clickable: bool,
    pub hoverable: bool,
    pub focusable: bool,
    pub clip_to_bounds: bool,
    pub control_id: Option<String>,
}

impl UiArrangedNode {
    pub fn effective_visibility(&self) -> UiVisibility {
        self.visibility
    }

    pub fn is_render_visible(&self) -> bool {
        self.effective_visibility().is_render_visible()
    }

    pub fn is_self_hit_test_visible(&self) -> bool {
        self.effective_visibility().allows_self_hit_test()
    }

    pub fn allows_child_hit_test(&self) -> bool {
        self.effective_visibility().allows_child_hit_test()
    }

    pub fn supports_pointer(&self) -> bool {
        self.enabled && (self.clickable || self.hoverable || self.focusable)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiArrangedTree {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: Vec<UiArrangedNode>,
    pub draw_order: Vec<UiNodeId>,
}

impl UiArrangedTree {
    pub fn get(&self, node_id: UiNodeId) -> Option<&UiArrangedNode> {
        self.nodes.iter().find(|node| node.node_id == node_id)
    }

    pub fn children_of(&self, node_id: UiNodeId) -> impl Iterator<Item = &UiArrangedNode> {
        self.get(node_id)
            .into_iter()
            .flat_map(|node| node.children.iter())
            .filter_map(|child_id| self.get(*child_id))
    }
}
