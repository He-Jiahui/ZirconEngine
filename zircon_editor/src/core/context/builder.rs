use std::sync::Arc;

use crate::core::commands::{CommandEvalSnapshotHandle, EditorCommandRegistryHandle};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::EditorTransactionEngine;
use crate::core::editor_event::EditorEventService;
use crate::core::editor_message::SharedEditorMessageBus;
use crate::core::jobs::{EditorJobLimits, EditorJobSystem};
use zircon_runtime::core::runtime::tasks::JobScheduler;

use super::EditorContext;

pub struct EditorContextBuilder {
    bus: SharedEditorMessageBus,
    scheduler: JobScheduler,
}

impl EditorContextBuilder {
    pub fn new(scheduler: JobScheduler) -> Self {
        Self {
            bus: SharedEditorMessageBus::default(),
            scheduler,
        }
    }

    pub fn with_bus(mut self, bus: SharedEditorMessageBus) -> Self {
        self.bus = bus;
        self
    }

    pub fn build(self) -> Arc<EditorContext> {
        let events = Arc::new(EditorEventService::new(self.bus.clone()));
        let jobs = EditorJobSystem::with_scheduler_and_bus(
            self.scheduler,
            self.bus.clone(),
            EditorJobLimits::default(),
        );
        let transactions = EditorTransactionEngine::new(CoreEditContext::default());
        let commands = EditorCommandRegistryHandle::default_workbench();
        let command_eval = CommandEvalSnapshotHandle::default();
        Arc::new(EditorContext::new(
            self.bus,
            events,
            jobs,
            transactions,
            commands,
            command_eval,
        ))
    }
}
