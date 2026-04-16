use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;

use crate::UiBindingRef;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiTemplateNodeMetadata {
    pub component: String,
    pub control_id: Option<String>,
    pub classes: Vec<String>,
    pub attributes: BTreeMap<String, Value>,
    pub slot_attributes: BTreeMap<String, Value>,
    pub style_overrides: BTreeMap<String, Value>,
    pub style_tokens: BTreeMap<String, String>,
    pub bindings: Vec<UiBindingRef>,
}
