use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiBehaviorNodeKind {
    #[default]
    Selector,
    Sequence,
    Parallel,
    Decorator,
    Service,
    Task,
    Subtree,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiBehaviorNodeDescriptor {
    pub id: String,
    pub kind: AiBehaviorNodeKind,
    pub display_name: String,
    pub children: Vec<String>,
}

impl AiBehaviorNodeDescriptor {
    pub fn new(
        id: impl Into<String>,
        kind: AiBehaviorNodeKind,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            display_name: display_name.into(),
            children: Vec::new(),
        }
    }

    pub fn with_child(mut self, child: impl Into<String>) -> Self {
        self.children.push(child.into());
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiBehaviorTreeDescriptor {
    pub id: String,
    pub display_name: String,
    pub root_node: String,
    pub nodes: Vec<AiBehaviorNodeDescriptor>,
}

impl AiBehaviorTreeDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        root_node: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            root_node: root_node.into(),
            nodes: Vec::new(),
        }
    }

    pub fn with_node(mut self, node: AiBehaviorNodeDescriptor) -> Self {
        self.nodes.push(node);
        self
    }
}
