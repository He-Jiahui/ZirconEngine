use zircon_runtime::scene::NodeId;

use crate::core::editor_event::{
    EditorEvent, EditorEventEnvelope, EditorEventSource, EditorHierarchyEvent,
};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::common::dispatch_envelope;

pub(crate) fn dispatch_hierarchy_reparent(
    runtime: &EditorHostEventController,
    node_ids: Vec<NodeId>,
    parent: Option<NodeId>,
) -> Result<UiHostEventEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::RetainedHost,
            EditorEvent::Hierarchy(EditorHierarchyEvent::ReparentNodes { node_ids, parent }),
        ),
    )
}

pub(crate) fn dispatch_hierarchy_rename(
    runtime: &EditorHostEventController,
    node_id: NodeId,
    name: String,
) -> Result<UiHostEventEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::RetainedHost,
            EditorEvent::Hierarchy(EditorHierarchyEvent::RenameNode { node_id, name }),
        ),
    )
}
