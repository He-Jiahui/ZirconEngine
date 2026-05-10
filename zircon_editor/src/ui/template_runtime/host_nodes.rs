use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::{binding::UiEventKind, event_ui::UiRouteId, layout::UiFrame};

#[derive(Clone, Debug, PartialEq)]
pub struct RetainedUiHostBindingProjection {
    pub binding_id: String,
    pub event_kind: UiEventKind,
    pub route_id: Option<UiRouteId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RetainedUiHostNodeProjection {
    pub node_id: String,
    pub parent_id: Option<String>,
    pub component: String,
    pub control_id: Option<String>,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub z_index: i32,
    pub attributes: BTreeMap<String, Value>,
    pub style_tokens: BTreeMap<String, String>,
    pub bindings: Vec<RetainedUiHostBindingProjection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RetainedUiHostModel {
    pub document_id: String,
    pub nodes: Vec<RetainedUiHostNodeProjection>,
}

impl RetainedUiHostModel {
    pub fn node(&self, node_id: &str) -> Option<&RetainedUiHostNodeProjection> {
        self.nodes.iter().find(|node| node.node_id == node_id)
    }

    pub fn node_by_control_id(&self, control_id: &str) -> Option<&RetainedUiHostNodeProjection> {
        self.nodes
            .iter()
            .find(|node| node.control_id.as_deref() == Some(control_id))
    }
}
