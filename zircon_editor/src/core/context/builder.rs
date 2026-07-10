use std::sync::Arc;

use crate::core::editor_event::EditorEventService;
use crate::core::editor_message::SharedEditorMessageBus;

use super::EditorContext;

pub struct EditorContextBuilder {
    bus: SharedEditorMessageBus,
}

impl EditorContextBuilder {
    pub fn new() -> Self {
        Self {
            bus: SharedEditorMessageBus::default(),
        }
    }

    pub fn with_bus(mut self, bus: SharedEditorMessageBus) -> Self {
        self.bus = bus;
        self
    }

    pub fn build(self) -> Arc<EditorContext> {
        let events = Arc::new(EditorEventService::new(self.bus.clone()));
        Arc::new(EditorContext::new(self.bus, events))
    }
}
