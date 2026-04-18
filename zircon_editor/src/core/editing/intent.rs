//! High-level user intents applied to editor state.

use zircon_scene::{NodeId, NodeKind};

#[derive(Clone, Debug)]
pub enum EditorIntent {
    CreateNode(NodeKind),
    DeleteNode(NodeId),
    SelectNode(NodeId),
    RenameNode(NodeId, String),
    SetParent(NodeId, Option<NodeId>),
    SetTransform(NodeId, zircon_math::Transform),
    ApplyInspectorChanges,
    BeginGizmoDrag,
    DragGizmo,
    EndGizmoDrag,
    Undo,
    Redo,
}
