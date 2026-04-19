use std::collections::BTreeMap;

use toml::Value;
use zircon_ui::{binding::UiEventKind, event_ui::UiRouteId, UiFrame};

#[derive(Clone, Debug, PartialEq)]
pub struct SlintUiHostBindingProjection {
    pub binding_id: String,
    pub event_kind: UiEventKind,
    pub route_id: Option<UiRouteId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SlintUiHostNodeProjection {
    pub node_id: String,
    pub parent_id: Option<String>,
    pub component: String,
    pub control_id: Option<String>,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub z_index: i32,
    pub attributes: BTreeMap<String, Value>,
    pub style_tokens: BTreeMap<String, String>,
    pub bindings: Vec<SlintUiHostBindingProjection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SlintUiHostModel {
    pub document_id: String,
    pub nodes: Vec<SlintUiHostNodeProjection>,
}

impl SlintUiHostModel {
    pub fn node(&self, node_id: &str) -> Option<&SlintUiHostNodeProjection> {
        self.nodes.iter().find(|node| node.node_id == node_id)
    }

    pub fn node_by_control_id(&self, control_id: &str) -> Option<&SlintUiHostNodeProjection> {
        self.nodes
            .iter()
            .find(|node| node.control_id.as_deref() == Some(control_id))
    }
}
