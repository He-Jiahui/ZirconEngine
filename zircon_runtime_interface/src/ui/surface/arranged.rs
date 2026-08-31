use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use crate::ui::layout::{
    UiAlignment2D, UiCanvasSlotPlacement, UiFrame, UiGridSlotPlacement, UiLinearSlotSizing,
    UiMargin, UiSlot, UiSlotKind,
};
use crate::ui::tree::{UiInputPolicy, UiPointerEvents, UiVisibility};

use super::UiPersistentSequence;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiArrangedSlotSummary {
    pub parent_id: UiNodeId,
    pub child_id: UiNodeId,
    pub kind: UiSlotKind,
    pub padding: UiMargin,
    pub alignment: UiAlignment2D,
    pub linear_sizing: Option<UiLinearSlotSizing>,
    pub canvas_placement: Option<UiCanvasSlotPlacement>,
    pub grid_placement: Option<UiGridSlotPlacement>,
    pub order: i32,
    pub z_order: i32,
    pub dirty_revision: u64,
}

impl From<&UiSlot> for UiArrangedSlotSummary {
    fn from(slot: &UiSlot) -> Self {
        Self {
            parent_id: slot.parent_id,
            child_id: slot.child_id,
            kind: slot.kind,
            padding: slot.padding,
            alignment: slot.alignment,
            linear_sizing: slot.linear_sizing,
            canvas_placement: slot.canvas_placement,
            grid_placement: slot.grid_placement,
            order: slot.order,
            z_order: slot.z_order,
            dirty_revision: slot.dirty_revision,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiCanvasLayerGroup {
    pub parent_id: UiNodeId,
    pub layer_index: u32,
    pub z_order: i32,
    pub child_ids: Vec<UiNodeId>,
}

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
    #[serde(default)]
    pub pointer_events: UiPointerEvents,
    pub enabled: bool,
    pub clickable: bool,
    pub hoverable: bool,
    pub focusable: bool,
    pub clip_to_bounds: bool,
    pub control_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<UiArrangedSlotSummary>,
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

    pub fn allows_self_pointer_hit_test(&self) -> bool {
        self.is_self_hit_test_visible() && self.pointer_events.allows_self_hit_test()
    }

    pub fn allows_child_pointer_hit_test(&self) -> bool {
        self.allows_child_hit_test() && self.pointer_events.allows_child_hit_test()
    }

    pub fn supports_pointer(&self) -> bool {
        self.enabled
            && self.allows_self_pointer_hit_test()
            && (self.clickable || self.hoverable || self.focusable)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiArrangedTree {
    pub tree_id: UiTreeId,
    pub roots: UiPersistentSequence<UiNodeId>,
    pub nodes: UiPersistentSequence<UiArrangedNode>,
    pub draw_order: UiPersistentSequence<UiNodeId>,
    #[serde(default)]
    pub canvas_layers: UiPersistentSequence<UiCanvasLayerGroup>,
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
