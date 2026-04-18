use serde::{Deserialize, Serialize};

use crate::ViewInstanceId;

use super::{SplitAxis, TabStackLayout};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DocumentNode {
    SplitNode {
        axis: SplitAxis,
        ratio: f32,
        first: Box<DocumentNode>,
        second: Box<DocumentNode>,
    },
    Tabs(TabStackLayout),
}

impl Default for DocumentNode {
    fn default() -> Self {
        Self::Tabs(TabStackLayout::default())
    }
}

impl DocumentNode {
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
}
