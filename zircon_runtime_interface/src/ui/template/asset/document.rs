use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::Value;

use super::{UiComponentApiVersion, UiComponentPublicContract, UiResourceRef};
use crate::ui::template::UiBindingRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiAssetKind {
    Layout,
    Widget,
    Style,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAssetHeader {
    pub kind: UiAssetKind,
    pub id: String,
    #[serde(default = "default_asset_version")]
    pub version: u32,
    #[serde(default)]
    pub display_name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAssetImports {
    #[serde(default)]
    pub widgets: Vec<String>,
    #[serde(default)]
    pub styles: Vec<String>,
    #[serde(default)]
    pub resources: Vec<UiResourceRef>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAssetDocument {
    pub asset: UiAssetHeader,
    #[serde(default)]
    pub imports: UiAssetImports,
    #[serde(default)]
    pub tokens: BTreeMap<String, Value>,
    #[serde(default)]
    pub root: Option<UiNodeDefinition>,
    #[serde(default)]
    pub components: BTreeMap<String, UiComponentDefinition>,
    #[serde(default)]
    pub stylesheets: Vec<UiStyleSheet>,
}

impl UiAssetDocument {
    pub fn root_node_id(&self) -> Option<&str> {
        self.root.as_ref().map(|node| node.node_id.as_str())
    }

    pub fn component_root_node_id(&self, component_name: &str) -> Option<&str> {
        self.components
            .get(component_name)
            .map(|component| component.root.node_id.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiNodeDefinitionKind {
    #[default]
    Native,
    Component,
    Reference,
    Slot,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiNodeDefinition {
    #[serde(default)]
    pub node_id: String,
    #[serde(default)]
    pub kind: UiNodeDefinitionKind,
    #[serde(default, rename = "type")]
    pub widget_type: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub component_ref: Option<String>,
    #[serde(default)]
    pub component_api_version: Option<UiComponentApiVersion>,
    #[serde(default)]
    pub slot_name: Option<String>,
    #[serde(default)]
    pub control_id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
    #[serde(default)]
    pub props: BTreeMap<String, Value>,
    #[serde(default)]
    pub layout: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    pub bindings: Vec<UiBindingRef>,
    #[serde(default)]
    pub style_overrides: UiStyleDeclarationBlock,
    #[serde(default)]
    pub children: Vec<UiChildMount>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiChildMount {
    #[serde(default)]
    pub mount: Option<String>,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
    pub node: UiNodeDefinition,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiComponentDefinition {
    pub root: UiNodeDefinition,
    #[serde(default)]
    pub style_scope: UiStyleScope,
    #[serde(default)]
    pub contract: UiComponentPublicContract,
    #[serde(default)]
    pub params: BTreeMap<String, UiComponentParamSchema>,
    #[serde(default)]
    pub slots: BTreeMap<String, UiNamedSlotSchema>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiComponentParamSchema {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub default: Option<Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNamedSlotSchema {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub multiple: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiStyleScope {
    Open,
    #[default]
    Closed,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiStyleSheet {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub rules: Vec<UiStyleRule>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiStyleRule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub selector: String,
    #[serde(default)]
    pub set: UiStyleDeclarationBlock,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiStyleDeclarationBlock {
    #[serde(default, rename = "self")]
    pub self_values: BTreeMap<String, Value>,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiActionRef {
    #[serde(default)]
    pub route: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub payload: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiAssetError {
    #[error("failed to parse ui asset document: {0}")]
    ParseToml(String),
    #[error("failed to read ui asset document: {0}")]
    Io(String),
    #[error("ui asset {asset_id} uses unsupported schema version {version}; current supported version is {current}")]
    UnsupportedSchemaVersion {
        asset_id: String,
        version: u32,
        current: u32,
    },
    #[error("ui asset {asset_id} schema migration failed: {detail}")]
    SchemaMigrationFailed { asset_id: String, detail: String },
    #[error("ui asset {asset_id} is invalid: {detail}")]
    InvalidDocument { asset_id: String, detail: String },
    #[error("ui asset {asset_id} references missing node {node_id}")]
    MissingNode { asset_id: String, node_id: String },
    #[error("ui asset {asset_id} references unknown component {component}")]
    UnknownComponent { asset_id: String, component: String },
    #[error("ui asset reference {reference} is not registered")]
    UnknownImport { reference: String },
    #[error("ui asset reference {reference} expected kind {expected:?} but received {actual:?}")]
    ImportKindMismatch {
        reference: String,
        expected: UiAssetKind,
        actual: UiAssetKind,
    },
    #[error("ui component {component} missing required slot {slot_name}")]
    MissingRequiredSlot {
        component: String,
        slot_name: String,
    },
    #[error("ui component {component} received unknown slot {slot_name}")]
    UnknownSlot {
        component: String,
        slot_name: String,
    },
    #[error("ui component {component} slot {slot_name} does not accept multiple children")]
    SlotDoesNotAcceptMultiple {
        component: String,
        slot_name: String,
    },
    #[error("ui selector is invalid: {0}")]
    InvalidSelector(String),
}

const fn default_asset_version() -> u32 {
    1
}
