use std::sync::Arc;

use crate::core::commands::{CommandEvalSnapshotHandle, EditorCommandRegistryHandle};
use crate::core::editing::engine::EditorTransactionEngine;
use crate::core::editor_event::EditorEventService;
use crate::core::editor_message::SharedEditorMessageBus;
use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::jobs::EditorJobSystem;

/// Explicit L1 editor service aggregate. Each service owns its own synchronization.
pub struct EditorContext {
    bus: SharedEditorMessageBus,
    events: Arc<EditorEventService>,
    jobs: EditorJobSystem,
    transactions: EditorTransactionEngine,
    commands: EditorCommandRegistryHandle,
    command_eval: CommandEvalSnapshotHandle,
    gateway: EditorRuntimeGatewayHandle,
}

impl EditorContext {
    pub(super) fn new(
        bus: SharedEditorMessageBus,
        events: Arc<EditorEventService>,
        jobs: EditorJobSystem,
        transactions: EditorTransactionEngine,
        commands: EditorCommandRegistryHandle,
        command_eval: CommandEvalSnapshotHandle,
        gateway: EditorRuntimeGatewayHandle,
    ) -> Self {
        Self {
            bus,
            events,
            jobs,
            transactions,
            commands,
            command_eval,
            gateway,
        }
    }

    pub fn bus(&self) -> &SharedEditorMessageBus {
        &self.bus
    }

    pub fn events(&self) -> &Arc<EditorEventService> {
        &self.events
    }

    pub fn jobs(&self) -> &EditorJobSystem {
        &self.jobs
    }

    pub fn transactions(&self) -> &EditorTransactionEngine {
        &self.transactions
    }

    pub fn commands(&self) -> &EditorCommandRegistryHandle {
        &self.commands
    }

    pub fn command_eval(&self) -> &CommandEvalSnapshotHandle {
        &self.command_eval
    }

    pub fn gateway(&self) -> &EditorRuntimeGatewayHandle {
        &self.gateway
    }
}
