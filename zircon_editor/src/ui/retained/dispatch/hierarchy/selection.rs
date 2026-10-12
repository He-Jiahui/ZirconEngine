use zircon_runtime::scene::NodeId;

use crate::core::editor_event::{
    EditorEvent, EditorEventEnvelope, EditorEventSource, SelectionHostEvent,
};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::common::dispatch_envelope;

pub(crate) fn dispatch_hierarchy_selection(
    runtime: &EditorHostEventController,
    node_id: NodeId,
) -> Result<UiHostEventEffects, String> {
    let world_domain = runtime.active_hierarchy_world_domain();
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::RetainedHost,
            EditorEvent::Selection(SelectionHostEvent::SelectSceneNode {
                world_domain,
                node_id,
            }),
        ),
    )
}

#[cfg(test)]
#[path = "tests/selection.rs"]
mod tests;
