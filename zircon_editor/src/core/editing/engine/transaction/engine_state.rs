use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Condvar, Mutex};

use super::{
    dirty_batch, ActiveOperationGroup, CommandBox, DetachedTransactionEventSink, DocumentId,
    EditCommandError, EditContext, EditWorldRoute, HistoryContextId, HistoryStore,
    TransactionEventSink, TransactionId,
};

const DEFAULT_HISTORY_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MergeMode {
    #[default]
    Disable,
    Ends,
    All,
}

pub(super) struct ActiveTransaction {
    pub(super) id: TransactionId,
    pub(super) history: HistoryContextId,
    pub(super) label: String,
    pub(super) timestamp_frame: u64,
    pub(super) commands: Vec<CommandBox>,
    pub(super) participants: BTreeSet<DocumentId>,
    pub(super) selection_before: super::SelectionSnapshot,
    pub(super) route: EditWorldRoute,
    pub(super) merge_mode: MergeMode,
    pub(super) root: bool,
}

pub(super) struct EngineState {
    // The context is taken out while client code runs so no engine lock crosses a callback.
    pub(super) context: Option<Box<dyn EditContext>>,
    pub(super) histories: HashMap<HistoryContextId, HistoryStore>,
    pub(super) history_generations: BTreeMap<HistoryContextId, u64>,
    pub(super) history_dirty: dirty_batch::HistoryDirtyJournal,
    pub(super) active: Vec<ActiveTransaction>,
    pub(super) operation_group: Option<ActiveOperationGroup>,
    pub(super) next_transaction: u64,
    pub(super) current_frame: u64,
    pub(super) history_capacity: usize,
    pub(super) operation: Option<&'static str>,
    // A failed recovery freezes mutation while retaining the active/history recovery state.
    pub(super) faulted: bool,
    pub(super) drop_error: Option<EditCommandError>,
}

pub struct EditorTransactionEngine {
    pub(super) state: Mutex<EngineState>,
    pub(super) operation_changed: Condvar,
    pub(super) save_token_lineage: Arc<()>,
    pub(super) event_sink: Arc<dyn TransactionEventSink>,
}

impl EditorTransactionEngine {
    pub fn new(context: impl EditContext + 'static) -> Self {
        Self::with_event_sink(context, Arc::new(DetachedTransactionEventSink))
    }

    pub fn with_capacity(
        context: impl EditContext + 'static,
        history_capacity: usize,
    ) -> Result<Self, EditCommandError> {
        if history_capacity == 0 {
            return Err(EditCommandError::InvalidHistoryCapacity);
        }
        Ok(Self::build(
            context,
            history_capacity,
            Arc::new(DetachedTransactionEventSink),
        ))
    }

    pub fn with_event_sink(
        context: impl EditContext + 'static,
        event_sink: Arc<dyn TransactionEventSink>,
    ) -> Self {
        Self::build(context, DEFAULT_HISTORY_CAPACITY, event_sink)
    }

    fn build(
        context: impl EditContext + 'static,
        history_capacity: usize,
        event_sink: Arc<dyn TransactionEventSink>,
    ) -> Self {
        Self {
            state: Mutex::new(EngineState {
                context: Some(Box::new(context)),
                histories: HashMap::new(),
                history_generations: BTreeMap::new(),
                history_dirty: dirty_batch::HistoryDirtyJournal::default(),
                active: Vec::new(),
                operation_group: None,
                next_transaction: 1,
                current_frame: 0,
                history_capacity,
                operation: None,
                faulted: false,
                drop_error: None,
            }),
            operation_changed: Condvar::new(),
            save_token_lineage: Arc::new(()),
            event_sink,
        }
    }

    pub fn set_frame(&self, frame: u64) -> Result<(), EditCommandError> {
        self.start_operation("set frame")?;
        let mut state = self.lock_state();
        state.current_frame = frame;
        self.clear_operation_locked(&mut state);
        Ok(())
    }

    pub fn take_drop_error(&self) -> Option<EditCommandError> {
        self.lock_state().drop_error.take()
    }

    pub fn with_context<T: 'static, R>(
        &self,
        inspect: impl FnOnce(&T) -> R,
    ) -> Result<Option<R>, EditCommandError> {
        self.start_operation("inspect edit context")?;
        {
            let mut state = self.lock_state();
            if !state.active.is_empty() {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::InvariantViolation {
                    invariant: "public context inspection requires no active transaction scope",
                });
            }
        }
        let context = self.take_context()?;
        let result = context.as_any().downcast_ref::<T>().map(inspect);
        self.finish_operation(context, false);
        Ok(result)
    }

    pub fn with_context_mut<T: 'static, R>(
        &self,
        inspect: impl FnOnce(&mut T) -> R,
    ) -> Result<Option<R>, EditCommandError> {
        self.start_operation("mutate edit context")?;
        {
            let mut state = self.lock_state();
            if !state.active.is_empty() {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::InvariantViolation {
                    invariant: "public context mutation requires no active transaction scope",
                });
            }
        }
        let mut context = self.take_context()?;
        let result = context.as_any_mut().downcast_mut::<T>().map(inspect);
        self.finish_operation(context, false);
        Ok(result)
    }
}

#[cfg(test)]
#[path = "engine_state/hash_index_tests.rs"]
mod hash_index_tests;
