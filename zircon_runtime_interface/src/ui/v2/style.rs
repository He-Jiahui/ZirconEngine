use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiV2StyleSheet {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub rules: Vec<UiV2StyleRule>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiV2StyleRule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub selector: String,
    #[serde(default)]
    pub set: UiV2StyleDeclarationBlock,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiV2StyleDeclarationBlock {
    #[serde(default, rename = "self")]
    pub self_values: BTreeMap<String, Value>,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiV2ResolvedStyleSheet {
    #[serde(default)]
    pub nodes: BTreeMap<String, UiV2ResolvedStyle>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiV2ResolvedStyle {
    #[serde(default)]
    pub self_values: BTreeMap<String, Value>,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
}

impl UiV2ResolvedStyle {
    pub fn merge_block(&mut self, block: &UiV2StyleDeclarationBlock) {
        self.self_values.extend(block.self_values.clone());
        self.slot.extend(block.slot.clone());
    }
}
