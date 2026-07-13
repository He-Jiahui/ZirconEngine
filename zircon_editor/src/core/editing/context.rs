use std::any::Any;

use super::engine::{EditCommandError, EditContext, SelectionSnapshot};

/// Headless-safe owner for the core edit state used by the transaction engine.
#[derive(Default)]
pub(crate) struct CoreEditContext {
    selection: SelectionSnapshot,
}

impl EditContext for CoreEditContext {
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
