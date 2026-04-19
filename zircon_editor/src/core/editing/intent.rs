//! High-level user intents applied to editor state.

use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::NodeId;

#[derive(Clone, Debug)]
pub enum EditorIntent {
    CreateNode(NodeKind),
    DeleteNode(NodeId),
    SelectNode(NodeId),
    RenameNode(NodeId, String),
    SetParent(NodeId, Option<NodeId>),
    SetTransform(NodeId, zircon_runtime::core::math::Transform),
    ApplyInspectorChanges,
    BeginGizmoDrag,
    DragGizmo,
    EndGizmoDrag,
    Undo,
    Redo,
}
