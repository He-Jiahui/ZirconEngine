use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::EditorEventTransient;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorTransientUiState {
    hovered_node: Option<String>,
    focused_node: Option<String>,
    pressed_nodes: BTreeSet<String>,
    resizing_drawers: BTreeSet<String>,
    dragging_view: Option<String>,
}

impl EditorTransientUiState {
    pub fn apply(&mut self, update: &EditorEventTransient) {
        match update {
            EditorEventTransient::HoverNode { node_path, hovered } => {
                if *hovered {
                    self.hovered_node = Some(node_path.clone());
                } else if self.hovered_node.as_deref() == Some(node_path.as_str()) {
                    self.hovered_node = None;
                }
            }
            EditorEventTransient::FocusNode { node_path } => {
                self.focused_node = Some(node_path.clone());
            }
            EditorEventTransient::PressNode { node_path, pressed } => {
                if *pressed {
                    self.pressed_nodes.insert(node_path.clone());
                } else {
                    self.pressed_nodes.remove(node_path);
                }
            }
            EditorEventTransient::SetDrawerResizing {
                drawer_id,
                resizing,
            } => {
                if *resizing {
                    self.resizing_drawers.insert(drawer_id.clone());
                } else {
                    self.resizing_drawers.remove(drawer_id);
                }
            }
            EditorEventTransient::BeginViewDrag { instance_id } => {
                self.dragging_view = Some(instance_id.clone());
            }
            EditorEventTransient::EndViewDrag => {
                self.dragging_view = None;
            }
        }
    }

    pub fn is_node_hovered(&self, node_path: &str) -> bool {
        self.hovered_node.as_deref() == Some(node_path)
    }

    pub fn is_node_focused(&self, node_path: &str) -> bool {
        self.focused_node.as_deref() == Some(node_path)
    }

    pub fn is_node_pressed(&self, node_path: &str) -> bool {
        self.pressed_nodes.contains(node_path)
    }

    pub fn is_drawer_resizing(&self, drawer_id: &str) -> bool {
        self.resizing_drawers.contains(drawer_id)
    }

    pub fn is_view_dragging(&self, instance_id: &str) -> bool {
        self.dragging_view.as_deref() == Some(instance_id)
    }
}
