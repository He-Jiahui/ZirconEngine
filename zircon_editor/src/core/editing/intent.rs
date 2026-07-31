//! High-level user intents applied to editor state.

use zircon_runtime::scene::NodeId;
use zircon_runtime::scene::components::NodeKind;

#[derive(Clone, Debug)]
pub enum EditorIntent {
    CreateNode(NodeKind),
    DeleteNode(NodeId),
    DeleteNodes(Vec<NodeId>),
    SelectNode(NodeId),
    RenameNode(NodeId, String),
    SetParent(NodeId, Option<NodeId>),
    SetParents(Vec<NodeId>, Option<NodeId>),
    SetTransform(NodeId, zircon_runtime_interface::math::Transform),
    ApplyInspectorChanges,
    Undo,
    Redo,
}
