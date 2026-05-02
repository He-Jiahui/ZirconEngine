use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;

use crate::ui::template::{UiNodeDefinition, UiNodeDefinitionKind, UiStyleDeclarationBlock};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiDefaultNodeTemplate {
    #[serde(default)]
    pub node_id_prefix: String,
    #[serde(default)]
    pub widget_type: String,
    #[serde(default)]
    pub control_id_prefix: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub props: BTreeMap<String, Value>,
    #[serde(default)]
    pub layout: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    pub slot_name: Option<String>,
}

impl UiDefaultNodeTemplate {
    pub fn native(widget_type: impl Into<String>) -> Self {
        let widget_type = widget_type.into();
        Self {
            node_id_prefix: normalize_prefix(&widget_type),
            control_id_prefix: Some(control_prefix(&widget_type)),
            widget_type,
            ..Self::default()
        }
    }

    pub fn with_node_id_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.node_id_prefix = prefix.into();
        self
    }

    pub fn with_control_id_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.control_id_prefix = Some(prefix.into());
        self
    }

    pub fn with_prop(mut self, name: impl Into<String>, value: Value) -> Self {
        let _ = self.props.insert(name.into(), value);
        self
    }

    pub fn with_props(mut self, props: BTreeMap<String, Value>) -> Self {
        self.props = props;
        self
    }

    pub fn with_layout(mut self, layout: BTreeMap<String, Value>) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.widget_type.trim().is_empty() || self.node_id_prefix.trim().is_empty()
    }

    pub fn instantiate(
        &self,
        node_id: impl Into<String>,
        control_id: Option<String>,
    ) -> UiNodeDefinition {
        UiNodeDefinition {
            node_id: node_id.into(),
            kind: UiNodeDefinitionKind::Native,
            widget_type: Some(self.widget_type.clone()),
            component: None,
            component_ref: None,
            component_api_version: None,
            slot_name: self.slot_name.clone(),
            control_id: control_id.or_else(|| self.control_id_prefix.clone()),
            classes: self.classes.clone(),
            params: BTreeMap::new(),
            props: self.props.clone(),
            layout: self.layout.clone(),
            bindings: Vec::new(),
            style_overrides: UiStyleDeclarationBlock::default(),
            children: Vec::new(),
        }
    }
}

fn normalize_prefix(value: &str) -> String {
    let normalized = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_ascii_lowercase();
    if normalized.is_empty() {
        "node".to_string()
    } else {
        normalized
    }
}

fn control_prefix(value: &str) -> String {
    let prefix = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    if prefix.is_empty() {
        "Node".to_string()
    } else {
        prefix
    }
}
