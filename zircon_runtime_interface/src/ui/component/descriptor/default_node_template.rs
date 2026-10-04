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
        let (node_id_prefix, control_id_prefix) = native_prefixes(&widget_type);
        Self {
            node_id_prefix,
            control_id_prefix: Some(control_id_prefix),
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

    /// 调用方传入的 control_id 优先于模板前缀；本方法只生成单个 Native 节点，
    /// 参数、绑定、样式覆盖与子节点保持为空。
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
            ..UiNodeDefinition::default()
        }
    }
}

// 节点前缀统一小写并将非 ASCII 字母数字折成下划线；控制 ID 前缀只保留原大小写 ASCII 字母数字，
// 两者的空结果分别回退为 node 与 Node。
fn native_prefixes(value: &str) -> (String, String) {
    let mut node_prefix = String::with_capacity(value.len());
    let mut control_prefix = String::with_capacity(value.len());

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            node_prefix.push(ch.to_ascii_lowercase());
            control_prefix.push(ch);
        } else {
            node_prefix.push('_');
        }
    }

    let start = node_prefix
        .as_bytes()
        .iter()
        .position(|byte| *byte != b'_')
        .unwrap_or(0);
    let end = node_prefix
        .as_bytes()
        .iter()
        .rposition(|byte| *byte != b'_')
        .map_or(start, |index| index.saturating_add(1));
    node_prefix.truncate(end);
    if start > 0 {
        node_prefix.drain(..start);
    }
    if node_prefix.is_empty() {
        node_prefix.push_str("node");
    }
    if control_prefix.is_empty() {
        control_prefix.push_str("Node");
    }

    (node_prefix, control_prefix)
}

#[cfg(test)]
#[path = "default_node_template/tests/prefix_performance_tests.rs"]
mod prefix_performance_tests;
