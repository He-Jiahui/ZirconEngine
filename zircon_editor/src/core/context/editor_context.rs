use std::sync::Arc;

use crate::core::editor_event::EditorEventService;
use crate::core::editor_message::SharedEditorMessageBus;

/// Explicit L1 editor service aggregate. Each service owns its own synchronization.
pub struct EditorContext {
    bus: SharedEditorMessageBus,
    events: Arc<EditorEventService>,
}

impl EditorContext {
    pub(super) fn new(bus: SharedEditorMessageBus, events: Arc<EditorEventService>) -> Self {
        Self { bus, events }
    }

    pub fn bus(&self) -> &SharedEditorMessageBus {
        &self.bus
    }

    pub fn events(&self) -> &Arc<EditorEventService> {
        &self.events
    }
}
