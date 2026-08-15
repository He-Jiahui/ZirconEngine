use std::sync::{Arc, RwLock};

use super::CommandEvalCtx;

/// The single interactive command-context snapshot owned by `EditorContext`.
#[derive(Clone, Debug, Default)]
pub struct CommandEvalSnapshotHandle {
    snapshot: Arc<RwLock<CommandEvalSnapshot>>,
}

#[derive(Clone, Debug)]
struct CommandEvalSnapshot {
    generation: u64,
    context: Arc<CommandEvalCtx>,
}

impl Default for CommandEvalSnapshot {
    fn default() -> Self {
        Self {
            generation: 0,
            context: Arc::new(CommandEvalCtx::default()),
        }
    }
}

impl CommandEvalSnapshotHandle {
    pub fn snapshot(&self) -> CommandEvalCtx {
        self.shared_snapshot().as_ref().clone()
    }

    pub fn shared_snapshot(&self) -> Arc<CommandEvalCtx> {
        self.shared_snapshot_with_generation().1
    }

    /// Reads the generation and context under one lock so consumers cannot pair mismatched state.
    pub fn snapshot_with_generation(&self) -> (u64, CommandEvalCtx) {
        let (generation, context) = self.shared_snapshot_with_generation();
        (generation, context.as_ref().clone())
    }

    pub fn shared_snapshot_with_generation(&self) -> (u64, Arc<CommandEvalCtx>) {
        let snapshot = self
            .snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        (snapshot.generation, Arc::clone(&snapshot.context))
    }

    pub fn generation(&self) -> u64 {
        self.snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .generation
    }

    /// Replaces the shared context only when its command semantics changed.
    pub fn replace(&self, context: CommandEvalCtx) -> bool {
        let mut snapshot = self
            .snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if snapshot.context.as_ref() == &context {
            return false;
        }
        snapshot.context = Arc::new(context);
        snapshot.generation = snapshot.generation.wrapping_add(1);
        true
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::CommandEvalSnapshotHandle;
    use crate::core::commands::{CommandEvalCtx, WhenClause};

    #[test]
    fn equivalent_context_reuses_the_command_eval_generation() {
        let handle = CommandEvalSnapshotHandle::default();

        assert_eq!(
            handle.snapshot_with_generation(),
            (0, CommandEvalCtx::default())
        );
        assert!(!handle.replace(CommandEvalCtx::default()));
        assert_eq!(handle.generation(), 0);
    }

    #[test]
    fn semantic_context_changes_advance_the_generation_once() {
        let handle = CommandEvalSnapshotHandle::default();
        let selected = CommandEvalCtx::interactive().with_selection_count(1);

        assert!(handle.replace(selected.clone()));
        assert_eq!(handle.snapshot_with_generation(), (1, selected.clone()));
        assert!(!handle.replace(selected));
        assert_eq!(handle.generation(), 1);
    }

    #[test]
    fn cloned_handles_share_one_versioned_snapshot() {
        let handle = CommandEvalSnapshotHandle::default();
        let clone = handle.clone();
        let project_open = CommandEvalCtx::interactive().with_project_open(true);

        assert!(clone.replace(project_open.clone()));
        assert_eq!(handle.snapshot_with_generation(), (1, project_open));
    }

    #[test]
    fn shared_snapshot_reuses_one_context_arc_until_semantics_change() {
        let handle = CommandEvalSnapshotHandle::default();

        let first = handle.shared_snapshot();
        let second = handle.shared_snapshot();
        assert!(Arc::ptr_eq(&first, &second));

        assert!(handle.replace(CommandEvalCtx::interactive().with_project_open(true)));
        let changed = handle.shared_snapshot();
        assert!(!Arc::ptr_eq(&first, &changed));
        assert!(WhenClause::ProjectOpen.eval(changed.as_ref()));
    }
}
