use std::collections::BTreeMap;

use crate::ui::EditorUiBinding;
use toml::Value;
use zircon_runtime::ui::event_ui::UiRouteId;

#[derive(Clone, Debug, PartialEq)]
pub struct SlintUiBindingProjection {
    pub binding_id: String,
    pub binding: EditorUiBinding,
    pub route_id: Option<UiRouteId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SlintUiNodeProjection {
    pub component: String,
    pub control_id: Option<String>,
    pub attributes: BTreeMap<String, Value>,
    pub style_tokens: BTreeMap<String, String>,
    pub binding_ids: Vec<String>,
    pub children: Vec<SlintUiNodeProjection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SlintUiProjection {
    pub document_id: String,
    pub root: SlintUiNodeProjection,
    pub bindings: Vec<SlintUiBindingProjection>,
}
