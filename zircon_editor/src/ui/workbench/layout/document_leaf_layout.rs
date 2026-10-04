use serde::{Deserialize, Serialize};

use super::{DocumentNodeId, TabStackLayout};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "DocumentLeafWire", into = "DocumentLeafWire")]
pub struct DocumentLeafLayout {
    pub node_id: DocumentNodeId,
    pub tab_stack: TabStackLayout,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentLeafWire {
    #[serde(default = "DocumentNodeId::nil")]
    node_id: DocumentNodeId,
    tabs: Vec<crate::ui::workbench::view::ViewInstanceId>,
    active_tab: Option<crate::ui::workbench::view::ViewInstanceId>,
}

impl From<DocumentLeafWire> for DocumentLeafLayout {
    fn from(wire: DocumentLeafWire) -> Self {
        Self {
            node_id: if wire.node_id.is_nil() {
                DocumentNodeId::migrated_ordinal(1)
            } else {
                wire.node_id
            },
            tab_stack: TabStackLayout {
                tabs: wire.tabs,
                active_tab: wire.active_tab,
            },
        }
    }
}

impl From<DocumentLeafLayout> for DocumentLeafWire {
    fn from(leaf: DocumentLeafLayout) -> Self {
        Self {
            node_id: leaf.node_id,
            tabs: leaf.tab_stack.tabs,
            active_tab: leaf.tab_stack.active_tab,
        }
    }
}

impl DocumentLeafLayout {
    pub fn new(tab_stack: TabStackLayout) -> Self {
        Self {
            node_id: DocumentNodeId::default(),
            tab_stack,
        }
    }
}

impl std::ops::Deref for DocumentLeafLayout {
    type Target = TabStackLayout;

    fn deref(&self) -> &Self::Target {
        &self.tab_stack
    }
}

impl std::ops::DerefMut for DocumentLeafLayout {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tab_stack
    }
}

#[cfg(test)]
#[path = "tests/document_leaf_layout_identity_tests.rs"]
mod identity_tests;
