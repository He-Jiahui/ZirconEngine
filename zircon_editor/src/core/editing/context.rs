use std::any::Any;

use super::engine::{EditCommandError, EditContext, SelectionSnapshot};
use crate::core::gateway::EditorRuntimeGatewayHandle;

/// Headless-safe owner for the core edit state used by the transaction engine.
pub(crate) struct CoreEditContext {
    selection: SelectionSnapshot,
    gateway: EditorRuntimeGatewayHandle,
}

impl CoreEditContext {
    pub(crate) fn new(gateway: EditorRuntimeGatewayHandle) -> Self {
        Self {
            selection: SelectionSnapshot::default(),
            gateway,
        }
    }
}

impl Default for CoreEditContext {
    fn default() -> Self {
        Self::new(EditorRuntimeGatewayHandle::detached())
    }
}

impl EditContext for CoreEditContext {
    fn runtime_gateway(&self) -> &EditorRuntimeGatewayHandle {
        &self.gateway
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        self.selection.clone()
    }

    fn restore_selection(&mut self, snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
        self.selection = snapshot.clone();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
