use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::Value;

use crate::ui::template::{UiBindingRef, UiTemplateError};

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
    pub fn validate_tree_authority(&self) -> Result<(), UiAssetError> {
        let mut seen = BTreeMap::new();
        if let Some(root) = &self.root {
            validate_node_tree(&self.asset.id, "root", root, &mut seen)?;
        }
        for (component_name, component) in &self.components {
            validate_node_tree(
                &self.asset.id,
                &format!("component {component_name}"),
                &component.root,
                &mut seen,
            )?;
        }
        Ok(())
    }

    pub fn root_node_id(&self) -> Option<&str> {
        self.root.as_ref().map(|node| node.node_id.as_str())
    }

    pub fn contains_node(&self, node_id: &str) -> bool {
        self.node(node_id).is_some()
    }

    pub fn node(&self, node_id: &str) -> Option<&UiNodeDefinition> {
        if let Some(root) = &self.root {
            if let Some(node) = find_node(root, node_id) {
                return Some(node);
            }
        }
        self.components
            .values()
            .find_map(|component| find_node(&component.root, node_id))
    }

    pub fn node_mut(&mut self, node_id: &str) -> Option<&mut UiNodeDefinition> {
        if let Some(root) = &mut self.root {
            if let Some(node) = find_node_mut(root, node_id) {
                return Some(node);
            }
        }
        self.components
            .values_mut()
            .find_map(|component| find_node_mut(&mut component.root, node_id))
    }

    pub fn component_root_node_id(&self, component_name: &str) -> Option<&str> {
        self.components
            .get(component_name)
            .map(|component| component.root.node_id.as_str())
    }

    pub fn child_index_in_parent(&self, child_id: &str) -> Option<(String, usize)> {
        self.parent_of(child_id)
            .map(|parent| (parent.parent_node_id.to_string(), parent.child_index))
    }

    pub fn child_mount(&self, child_id: &str) -> Option<&UiChildMount> {
        if let Some(root) = &self.root {
            if let Some(mount) = find_child_mount(root, child_id) {
                return Some(mount);
            }
        }
        self.components
            .values()
            .find_map(|component| find_child_mount(&component.root, child_id))
    }

    pub fn child_mount_mut(&mut self, child_id: &str) -> Option<&mut UiChildMount> {
        if let Some(root) = &mut self.root {
            if let Some(mount) = find_child_mount_mut(root, child_id) {
                return Some(mount);
            }
        }
        self.components
            .values_mut()
            .find_map(|component| find_child_mount_mut(&mut component.root, child_id))
    }

    pub fn parent_of(&self, child_id: &str) -> Option<UiNodeParent<'_>> {
        if let Some(root) = &self.root {
            if let Some(parent) = find_parent(root, child_id) {
                return Some(parent);
            }
        }
        self.components
            .values()
            .find_map(|component| find_parent(&component.root, child_id))
    }

    pub fn iter_nodes(&self) -> UiAssetNodeIter<'_> {
        let mut stack = Vec::new();
        for component in self.components.values().rev() {
            stack.push(&component.root);
        }
        if let Some(root) = &self.root {
            stack.push(root);
        }
        UiAssetNodeIter { stack }
    }

    pub fn node_map(&self) -> BTreeMap<String, UiNodeDefinition> {
        self.iter_nodes()
            .map(|node| (node.node_id.clone(), node.clone()))
            .collect()
    }

    pub fn replace_node(&mut self, node_id: &str, replacement: UiNodeDefinition) -> bool {
        if self
            .root
            .as_ref()
            .is_some_and(|root| root.node_id == node_id)
        {
            if self.root.as_ref() == Some(&replacement) {
                return false;
            }
            self.root = Some(replacement);
            return true;
        }
        if let Some(root) = &mut self.root {
            if replace_node_in_tree(root, node_id, &replacement) {
                return true;
            }
        }
        for component in self.components.values_mut() {
            if component.root.node_id == node_id {
                if component.root == replacement {
                    return false;
                }
                component.root = replacement;
                return true;
            }
            if replace_node_in_tree(&mut component.root, node_id, &replacement) {
                return true;
            }
        }
        false
    }

    pub fn remove_node(&mut self, node_id: &str) -> Option<UiNodeDefinition> {
        if self
            .root
            .as_ref()
            .is_some_and(|root| root.node_id == node_id)
        {
            return self.root.take();
        }
        if let Some(root) = &mut self.root {
            if let Some(removed) = remove_node_from_tree(root, node_id) {
                return Some(removed);
            }
        }
        for component in self.components.values_mut() {
            if let Some(removed) = remove_node_from_tree(&mut component.root, node_id) {
                return Some(removed);
            }
        }
        None
    }

    pub fn insert_child(&mut self, parent_id: &str, index: usize, child: UiChildMount) -> bool {
        let Some(parent) = self.node_mut(parent_id) else {
            return false;
        };
        let insert_index = index.min(parent.children.len());
        parent.children.insert(insert_index, child);
        true
    }

    pub fn push_child(&mut self, parent_id: &str, child: UiChildMount) -> bool {
        let Some(parent) = self.node_mut(parent_id) else {
            return false;
        };
        parent.children.push(child);
        true
    }

    pub fn swap_children(&mut self, parent_id: &str, left: usize, right: usize) -> bool {
        let Some(parent) = self.node_mut(parent_id) else {
            return false;
        };
        if left >= parent.children.len() || right >= parent.children.len() || left == right {
            return false;
        }
        parent.children.swap(left, right);
        true
    }
}

pub struct UiAssetNodeIter<'a> {
    stack: Vec<&'a UiNodeDefinition>,
}

impl<'a> Iterator for UiAssetNodeIter<'a> {
    type Item = &'a UiNodeDefinition;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        for child in node.children.iter().rev() {
            self.stack.push(&child.node);
        }
        Some(node)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiNodeParent<'a> {
    pub parent_node_id: &'a str,
    pub child_index: usize,
    pub mount: Option<&'a str>,
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
    #[error("ui asset legacy adapter failed: {0}")]
    LegacyTemplate(String),
}

impl From<UiTemplateError> for UiAssetError {
    fn from(value: UiTemplateError) -> Self {
        Self::LegacyTemplate(value.to_string())
    }
}

const fn default_asset_version() -> u32 {
    1
}

fn validate_node_tree(
    asset_id: &str,
    scope: &str,
    node: &UiNodeDefinition,
    seen: &mut BTreeMap<String, UiNodeDefinition>,
) -> Result<(), UiAssetError> {
    if node.node_id.trim().is_empty() {
        return Err(UiAssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: format!("{scope} contains a node with an empty node_id"),
        });
    }
    if let Some(existing) = seen.get(&node.node_id) {
        if existing == node {
            return Ok(());
        }
        return Err(UiAssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: format!(
                "duplicate node_id {} resolves to conflicting subtrees",
                node.node_id
            ),
        });
    }
    let _ = seen.insert(node.node_id.clone(), node.clone());
    for child in &node.children {
        validate_node_tree(asset_id, scope, &child.node, seen)?;
    }
    Ok(())
}

fn find_node<'a>(node: &'a UiNodeDefinition, target: &str) -> Option<&'a UiNodeDefinition> {
    if node.node_id == target {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node(&child.node, target))
}

fn find_node_mut<'a>(
    node: &'a mut UiNodeDefinition,
    target: &str,
) -> Option<&'a mut UiNodeDefinition> {
    if node.node_id == target {
        return Some(node);
    }
    for child in &mut node.children {
        if let Some(found) = find_node_mut(&mut child.node, target) {
            return Some(found);
        }
    }
    None
}

fn find_child_mount<'a>(node: &'a UiNodeDefinition, child_id: &str) -> Option<&'a UiChildMount> {
    for child in &node.children {
        if child.node.node_id == child_id {
            return Some(child);
        }
        if let Some(found) = find_child_mount(&child.node, child_id) {
            return Some(found);
        }
    }
    None
}

fn find_child_mount_mut<'a>(
    node: &'a mut UiNodeDefinition,
    child_id: &str,
) -> Option<&'a mut UiChildMount> {
    for child in &mut node.children {
        if child.node.node_id == child_id {
            return Some(child);
        }
        if let Some(found) = find_child_mount_mut(&mut child.node, child_id) {
            return Some(found);
        }
    }
    None
}

fn find_parent<'a>(node: &'a UiNodeDefinition, child_id: &str) -> Option<UiNodeParent<'a>> {
    for (index, child) in node.children.iter().enumerate() {
        if child.node.node_id == child_id {
            return Some(UiNodeParent {
                parent_node_id: node.node_id.as_str(),
                child_index: index,
                mount: child.mount.as_deref(),
            });
        }
        if let Some(parent) = find_parent(&child.node, child_id) {
            return Some(parent);
        }
    }
    None
}

fn replace_node_in_tree(
    node: &mut UiNodeDefinition,
    target: &str,
    replacement: &UiNodeDefinition,
) -> bool {
    for child in &mut node.children {
        if child.node.node_id == target {
            if child.node == *replacement {
                return false;
            }
            child.node = replacement.clone();
            return true;
        }
        if replace_node_in_tree(&mut child.node, target, replacement) {
            return true;
        }
    }
    false
}

fn remove_node_from_tree(node: &mut UiNodeDefinition, target: &str) -> Option<UiNodeDefinition> {
    if let Some(index) = node
        .children
        .iter()
        .position(|child| child.node.node_id == target)
    {
        return Some(node.children.remove(index).node);
    }
    for child in &mut node.children {
        if let Some(removed) = remove_node_from_tree(&mut child.node, target) {
            return Some(removed);
        }
    }
    None
}
