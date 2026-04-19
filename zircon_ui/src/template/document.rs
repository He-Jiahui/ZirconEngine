use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::binding::UiEventKind;

use super::UiActionRef;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiTemplateDocument {
    #[serde(default = "default_template_version")]
    pub version: u32,
    #[serde(default)]
    pub components: BTreeMap<String, UiComponentTemplate>,
    pub root: UiTemplateNode,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiComponentTemplate {
    #[serde(default)]
    pub slots: BTreeMap<String, UiSlotTemplate>,
    pub root: UiTemplateNode,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSlotTemplate {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub multiple: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBindingRef {
    pub id: String,
    pub event: UiEventKind,
    #[serde(default)]
    pub route: Option<String>,
    #[serde(default)]
    pub action: Option<UiActionRef>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiTemplateNode {
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub slot: Option<String>,
    #[serde(default)]
    pub control_id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub bindings: Vec<UiBindingRef>,
    #[serde(default)]
    pub children: Vec<UiTemplateNode>,
    #[serde(default)]
    pub slots: BTreeMap<String, Vec<UiTemplateNode>>,
    #[serde(default)]
    pub attributes: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub slot_attributes: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub style_overrides: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub style_tokens: BTreeMap<String, String>,
}

impl UiTemplateNode {
    pub fn node_kind_count(&self) -> usize {
        usize::from(self.component.is_some())
            + usize::from(self.template.is_some())
            + usize::from(self.slot.is_some())
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiTemplateError {
    #[error("failed to parse ui template document: {0}")]
    ParseToml(String),
    #[error("failed to read ui template document: {0}")]
    Io(String),
    #[error("ui template node is invalid: {detail}")]
    InvalidNodeDefinition { detail: String },
    #[error("ui template references unknown component {template_id}")]
    UnknownTemplate { template_id: String },
    #[error("ui template {template_id} missing required slot {slot_name}")]
    MissingRequiredSlot {
        template_id: String,
        slot_name: String,
    },
    #[error("ui template {template_id} received unknown slot {slot_name}")]
    UnknownSlot {
        template_id: String,
        slot_name: String,
    },
    #[error("ui template {template_id} slot {slot_name} does not accept multiple children")]
    SlotDoesNotAcceptMultiple {
        template_id: String,
        slot_name: String,
    },
    #[error("ui template {template_id} uses undeclared slot placeholder {slot_name}")]
    UndeclaredSlotPlaceholder {
        template_id: String,
        slot_name: String,
    },
}

const fn default_template_version() -> u32 {
    1
}
