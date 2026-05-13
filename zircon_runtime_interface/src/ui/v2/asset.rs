use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::Value;

use crate::ui::template::{
    UiAssetImports, UiBindingRef, UiComponentParamSchema, UiNamedSlotSchema, UiStyleScope,
};

use super::{UiV2StyleDeclarationBlock, UiV2StyleSheet};

pub const UI_V2_ASSET_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiV2AssetKind {
    View,
    Component,
    Style,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiV2AssetHeader {
    pub kind: UiV2AssetKind,
    pub id: String,
    #[serde(default = "default_v2_asset_version")]
    pub version: u32,
    #[serde(default)]
    pub display_name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiV2AssetDocument {
    pub asset: UiV2AssetHeader,
    #[serde(default)]
    pub imports: UiAssetImports,
    #[serde(default)]
    pub tokens: BTreeMap<String, Value>,
    #[serde(default)]
    pub root: Option<UiV2Root>,
    #[serde(default)]
    pub nodes: BTreeMap<String, UiV2NodeDefinition>,
    #[serde(default)]
    pub components: BTreeMap<String, UiV2ComponentDefinition>,
    #[serde(default)]
    pub stylesheets: Vec<UiV2StyleSheet>,
}

impl UiV2AssetDocument {
    pub fn root_node_id(&self) -> Option<&str> {
        self.root.as_ref().map(|root| root.node.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiV2Root {
    pub node: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiV2NodeDefinition {
    pub component: String,
    #[serde(default)]
    pub control_id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub props: BTreeMap<String, Value>,
    #[serde(default)]
    pub state: BTreeMap<String, Value>,
    #[serde(default)]
    pub layout: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    pub style: UiV2StyleDeclarationBlock,
    #[serde(default)]
    pub slots: BTreeMap<String, Value>,
    #[serde(default)]
    pub events: Vec<UiBindingRef>,
    #[serde(default)]
    pub children: Vec<UiV2ChildMount>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiV2ChildMount {
    pub node: String,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiV2ComponentDefinition {
    pub root: String,
    #[serde(default)]
    pub style_scope: UiStyleScope,
    #[serde(default)]
    pub params: BTreeMap<String, UiComponentParamSchema>,
    #[serde(default)]
    pub slots: BTreeMap<String, UiNamedSlotSchema>,
    #[serde(default)]
    pub default_classes: Vec<String>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiV2AssetError {
    #[error("failed to parse ui v2 asset document: {0}")]
    ParseToml(String),
    #[error("failed to read ui v2 asset document: {0}")]
    Io(String),
    #[error(
        "ui v2 asset {asset_id} uses unsupported schema version {version}; expected {expected}"
    )]
    UnsupportedSchemaVersion {
        asset_id: String,
        version: u32,
        expected: u32,
    },
    #[error("ui v2 asset {asset_id} is invalid: {detail}")]
    InvalidDocument { asset_id: String, detail: String },
    #[error("ui v2 asset {asset_id} references missing node {node_id}")]
    MissingNode { asset_id: String, node_id: String },
    #[error("ui v2 asset {asset_id} references unknown component {component}")]
    UnknownComponent { asset_id: String, component: String },
    #[error("ui v2 asset {asset_id} style selector is invalid: {selector}")]
    InvalidSelector { asset_id: String, selector: String },
    #[error("ui v2 component {component} in asset {asset_id} missing required slot {slot_name}")]
    MissingRequiredSlot {
        asset_id: String,
        component: String,
        slot_name: String,
    },
    #[error("ui v2 component {component} in asset {asset_id} received unknown slot {slot_name}")]
    UnknownSlot {
        asset_id: String,
        component: String,
        slot_name: String,
    },
    #[error(
        "ui v2 component {component} in asset {asset_id} slot {slot_name} does not accept multiple children"
    )]
    SlotDoesNotAcceptMultiple {
        asset_id: String,
        component: String,
        slot_name: String,
    },
}

const fn default_v2_asset_version() -> u32 {
    UI_V2_ASSET_SCHEMA_VERSION
}
