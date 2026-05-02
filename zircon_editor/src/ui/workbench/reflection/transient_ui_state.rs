use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::json;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeDescriptor, event_ui::UiPropertyDescriptor, event_ui::UiReflectionSnapshot,
    event_ui::UiValueType,
};

use crate::core::editor_event::EditorEventTransient;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct EditorTransientUiState {
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

pub(crate) fn apply_transient_projection(
    snapshot: &mut UiReflectionSnapshot,
    transient: &EditorTransientUiState,
) {
    for node in snapshot.nodes.values_mut() {
        let node_path = node.node_path.0.clone();
        let hovered = transient.is_node_hovered(&node_path);
        let focused = transient.is_node_focused(&node_path);
        let pressed = transient.is_node_pressed(&node_path);
        let resizing = drawer_id_from_path(&node_path)
            .is_some_and(|drawer_id| transient.is_drawer_resizing(drawer_id));
        let dragging = node_path
            .rsplit('/')
            .next()
            .is_some_and(|segment| transient.is_view_dragging(segment));

        node.state_flags.pressed = pressed;
        upsert_property(node, "transient.hovered", hovered);
        upsert_property(node, "transient.focused", focused);
        upsert_property(node, "transient.resizing", resizing);
        upsert_property(node, "transient.dragging", dragging);
    }
}

fn upsert_property(node: &mut UiNodeDescriptor, name: &str, value: bool) {
    node.properties.insert(
        name.to_string(),
        UiPropertyDescriptor::new(name, UiValueType::Bool, json!(value)),
    );
}

fn drawer_id_from_path(node_path: &str) -> Option<&str> {
    let prefix = "editor/workbench/drawers/";
    let remainder = node_path.strip_prefix(prefix)?;
    remainder.split('/').next()
}
