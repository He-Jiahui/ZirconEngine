use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewInstanceId;

use super::document_node_id::DocumentNodeIdRepair;
use super::{DocumentLeafLayout, DocumentNodeId, SplitAxis, TabStackLayout};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum DocumentNode {
    SplitNode {
        node_id: DocumentNodeId,
        axis: SplitAxis,
        ratio: f32,
        first: Box<DocumentNode>,
        second: Box<DocumentNode>,
    },
    Tabs(DocumentLeafLayout),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
enum DocumentNodeWire {
    SplitNode {
        #[serde(default = "DocumentNodeId::nil")]
        node_id: DocumentNodeId,
        axis: SplitAxis,
        ratio: f32,
        first: Box<DocumentNodeWire>,
        second: Box<DocumentNodeWire>,
    },
    Tabs(DocumentLeafLayout),
}

impl DocumentNodeWire {
    fn into_node(self) -> DocumentNode {
        match self {
            Self::SplitNode {
                node_id,
                axis,
                ratio,
                first,
                second,
            } => DocumentNode::SplitNode {
                node_id,
                axis,
                ratio,
                first: Box::new(first.into_node()),
                second: Box::new(second.into_node()),
            },
            Self::Tabs(leaf) => DocumentNode::Tabs(leaf),
        }
    }
}

impl<'de> Deserialize<'de> for DocumentNode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut node = DocumentNodeWire::deserialize(deserializer)?.into_node();
        let mut reserved = HashSet::new();
        node.collect_node_ids(&mut reserved);
        node.normalize_node_ids(&mut DocumentNodeIdRepair::new(reserved));
        Ok(node)
    }
}

impl Default for DocumentNode {
    fn default() -> Self {
        Self::tabs(TabStackLayout::default())
    }
}

impl DocumentNode {
    pub fn tabs(stack: TabStackLayout) -> Self {
        Self::Tabs(DocumentLeafLayout::new(stack))
    }

    pub fn node_id(&self) -> DocumentNodeId {
        match self {
            Self::SplitNode { node_id, .. } => *node_id,
            Self::Tabs(leaf) => leaf.node_id,
        }
    }

    pub(crate) fn collect_node_ids(&self, reserved: &mut HashSet<DocumentNodeId>) {
        let node_id = self.node_id();
        if !node_id.is_nil() && !node_id.is_migrated() {
            reserved.insert(node_id);
        }
        if let Self::SplitNode { first, second, .. } = self {
            first.collect_node_ids(reserved);
            second.collect_node_ids(reserved);
        }
    }

    pub(crate) fn normalize_node_ids(&mut self, repair: &mut DocumentNodeIdRepair) {
        let node_id = match self {
            Self::SplitNode { node_id, .. } => node_id,
            Self::Tabs(leaf) => &mut leaf.node_id,
        };
        repair.normalize(node_id);
        if let Self::SplitNode { first, second, .. } = self {
            first.normalize_node_ids(repair);
            second.normalize_node_ids(repair);
        }
    }

    pub(crate) fn node_at_path_mut(&mut self, path: &[usize]) -> Option<&mut DocumentNode> {
        if path.is_empty() {
            return Some(self);
        }

        match self {
            Self::Tabs(_) if path.len() == 1 && path[0] == 0 => Some(self),
            Self::SplitNode { first, second, .. } => match path[0] {
                0 => first.node_at_path_mut(&path[1..]),
                1 => second.node_at_path_mut(&path[1..]),
                _ => None,
            },
            Self::Tabs(_) => None,
        }
    }

    pub(crate) fn remove_instance(&mut self, instance_id: &ViewInstanceId) -> bool {
        match self {
            Self::Tabs(stack) => stack.remove(instance_id),
            Self::SplitNode { first, second, .. } => {
                first.remove_instance(instance_id) || second.remove_instance(instance_id)
            }
        }
    }

    pub(crate) fn contains(&self, instance_id: &ViewInstanceId) -> bool {
        match self {
            Self::Tabs(stack) => stack.tabs.contains(instance_id),
            Self::SplitNode { first, second, .. } => {
                first.contains(instance_id) || second.contains(instance_id)
            }
        }
    }

    pub(crate) fn instance_count(&self) -> usize {
        match self {
            Self::Tabs(stack) => stack.tabs.len(),
            Self::SplitNode { first, second, .. } => first
                .instance_count()
                .saturating_add(second.instance_count()),
        }
    }

    pub(crate) fn append_instance_ids(&self, out: &mut Vec<ViewInstanceId>) {
        match self {
            Self::Tabs(stack) => out.extend(stack.tabs.iter().cloned()),
            Self::SplitNode { first, second, .. } => {
                first.append_instance_ids(out);
                second.append_instance_ids(out);
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/document_node_identity_tests.rs"]
mod identity_tests;
