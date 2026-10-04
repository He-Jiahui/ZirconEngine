use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ui::event_ui::{UiNodeId, UiTreeId};

/// Opaque, surface-local capability for resolving the latest value of a secure text field.
///
/// The reference carries no text. A consumer must present it back to the owning Runtime UI
/// surface, which rejects forged, stale, cross-tree, or non-secure references.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSecureTextValueRef {
    tree_id: UiTreeId,
    node_id: UiNodeId,
    property: String,
    token: Uuid,
}

impl fmt::Debug for UiSecureTextValueRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UiSecureTextValueRef")
            .field("tree_id", &self.tree_id)
            .field("node_id", &self.node_id)
            .field("property", &self.property)
            .field("token", &"<redacted>")
            .finish()
    }
}

impl UiSecureTextValueRef {
    /// Issues a new opaque identity. Only a surface that registers the returned reference can
    /// resolve it.
    pub fn issue(tree_id: UiTreeId, node_id: UiNodeId, property: impl Into<String>) -> Self {
        Self {
            tree_id,
            node_id,
            property: property.into(),
            token: Uuid::new_v4(),
        }
    }

    pub fn tree_id(&self) -> &UiTreeId {
        &self.tree_id
    }

    pub const fn node_id(&self) -> UiNodeId {
        self.node_id
    }

    pub fn property(&self) -> &str {
        &self.property
    }
}

#[cfg(test)]
#[path = "tests/secure_text.rs"]
mod tests;
