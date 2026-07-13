use std::sync::{Arc, RwLock};

use super::CommandEvalCtx;

/// The single interactive command-context snapshot owned by `EditorContext`.
#[derive(Clone, Debug, Default)]
pub struct CommandEvalSnapshotHandle {
    snapshot: Arc<RwLock<CommandEvalCtx>>,
}

impl CommandEvalSnapshotHandle {
    pub fn snapshot(&self) -> CommandEvalCtx {
        self.snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn replace(&self, snapshot: CommandEvalCtx) {
        *self
            .snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = snapshot;
    }
}
