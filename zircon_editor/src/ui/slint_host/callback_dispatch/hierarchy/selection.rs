use zircon_runtime::scene::NodeId;

use crate::core::editor_event::{
    EditorEvent, EditorEventEnvelope, EditorEventRuntime, EditorEventSource, SelectionHostEvent,
};
use crate::ui::slint_host::event_bridge::UiHostEventEffects;

use super::super::common::dispatch_envelope;

pub(crate) fn dispatch_hierarchy_selection(
    runtime: &EditorEventRuntime,
    node_id: NodeId,
) -> Result<UiHostEventEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::Slint,
            EditorEvent::Selection(SelectionHostEvent::SelectSceneNode { node_id }),
        ),
    )
}
