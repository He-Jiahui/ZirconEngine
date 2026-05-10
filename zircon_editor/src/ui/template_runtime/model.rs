use std::collections::BTreeMap;

use crate::ui::binding::EditorUiBinding;
use toml::Value;
use zircon_runtime_interface::ui::event_ui::UiRouteId;

#[derive(Clone, Debug, PartialEq)]
pub struct RetainedUiBindingProjection {
    pub binding_id: String,
    pub binding: EditorUiBinding,
    pub route_id: Option<UiRouteId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RetainedUiNodeProjection {
    pub component: String,
    pub control_id: Option<String>,
    pub attributes: BTreeMap<String, Value>,
    pub style_tokens: BTreeMap<String, String>,
    pub binding_ids: Vec<String>,
    pub children: Vec<RetainedUiNodeProjection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RetainedUiProjection {
    pub document_id: String,
    pub root: RetainedUiNodeProjection,
    pub bindings: Vec<RetainedUiBindingProjection>,
}
