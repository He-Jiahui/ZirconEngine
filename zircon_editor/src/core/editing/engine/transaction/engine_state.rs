use crate::core::editing::engine::retention::{
    HistoryRetentionDomainV1, HistoryRetentionSnapshotV1, RetainedBoundBuilderV1, RetainedBytesV1,
    RetentionContractError, RootRetentionV1,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Condvar, Mutex, Weak};
use zircon_runtime_interface::retained_memory::FreshHashStorageV1;

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
    pub(super) selection_ceiling_v1: u64,
    pub(super) retention: RootRetentionV1,
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
    // Successful applies (including merged commands) and cancellations invalidate pending decisions.
    pub(super) applied_command_revision: u64,
    pub(super) history_capacity: usize,
    pub(super) operation: Option<&'static str>,
    pub(super) close_admission: Weak<()>,
    // A failed recovery freezes mutation while retaining the active/history recovery state.
    pub(super) faulted: bool,
    pub(super) drop_error: Option<EditCommandError>,
    pub(super) history_storage_v1: FreshHashStorageV1,
}

pub struct EditorTransactionEngine {
    pub(super) state: Mutex<EngineState>,
    pub(super) operation_changed: Condvar,
    pub(super) save_token_lineage: Arc<()>,
    pub(super) event_sink: Arc<dyn TransactionEventSink>,
    pub(super) retention_domain: HistoryRetentionDomainV1,
    // Last field: its lease outlives every history/container member above it.
    pub(super) retention_metadata: RootRetentionV1,
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
        Self::build(
            context,
            history_capacity,
            Arc::new(DetachedTransactionEventSink),
            HistoryRetentionDomainV1::shared_default()?,
        )
    }

    pub fn with_event_sink(
        context: impl EditContext + 'static,
        event_sink: Arc<dyn TransactionEventSink>,
    ) -> Self {
        Self::build(
            context,
            DEFAULT_HISTORY_CAPACITY,
            event_sink,
            HistoryRetentionDomainV1::shared_default()
                .expect("admitted default history retention model"),
        )
        .expect("default history metadata fits its shared retention domain")
    }

    fn build(
        context: impl EditContext + 'static,
        history_capacity: usize,
        event_sink: Arc<dyn TransactionEventSink>,
        retention_domain: HistoryRetentionDomainV1,
    ) -> Result<Self, EditCommandError> {
        let history_storage_v1 =
            FreshHashStorageV1::for_request::<(HistoryContextId, HistoryStore)>(0)?;
        let mut baseline = RetainedBoundBuilderV1::new(retention_domain.model());
        baseline.layout(std::mem::size_of::<Self>(), std::mem::align_of::<Self>())?;
        baseline.layout(
            std::mem::size_of::<EngineState>(),
            std::mem::align_of::<EngineState>(),
        )?;
        baseline.array::<(u64, HistoryContextId)>(4096)?;
        baseline.btree::<HistoryContextId, u64>(1)?;
        baseline.add(48)?; // save-token lineage and one owned close-admission Arc allocation
        let retention_metadata =
            RootRetentionV1::metadata(&retention_domain, baseline.bytes().get())?;
        Ok(Self {
            state: Mutex::new(EngineState {
                context: Some(Box::new(context)),
                histories: HashMap::new(),
                history_generations: BTreeMap::new(),
                history_dirty: dirty_batch::HistoryDirtyJournal::default(),
                active: Vec::new(),
                operation_group: None,
                next_transaction: 1,
                current_frame: 0,
                applied_command_revision: 0,
                history_capacity,
                operation: None,
                close_admission: Weak::new(),
                faulted: false,
                drop_error: None,
                history_storage_v1,
            }),
            operation_changed: Condvar::new(),
            save_token_lineage: Arc::new(()),
            event_sink,
            retention_domain,
            retention_metadata,
        })
    }
    pub fn try_new(context: impl EditContext + 'static) -> Result<Self, EditCommandError> {
        Self::build(
            context,
            DEFAULT_HISTORY_CAPACITY,
            Arc::new(DetachedTransactionEventSink),
            HistoryRetentionDomainV1::shared_default()?,
        )
    }
    pub fn with_retention_domain_v1(
        context: impl EditContext + 'static,
        history_capacity: usize,
        domain: HistoryRetentionDomainV1,
    ) -> Result<Self, EditCommandError> {
        if history_capacity == 0 {
            return Err(EditCommandError::InvalidHistoryCapacity);
        }
        Self::build(
            context,
            history_capacity,
            Arc::new(DetachedTransactionEventSink),
            domain,
        )
    }
    pub fn retention_snapshot_v1(&self) -> Result<HistoryRetentionSnapshotV1, EditCommandError> {
        Ok(self.retention_domain.snapshot()?)
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
        self.start_read_operation("inspect edit context")?;
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
#[path = "tests/hash_index.rs"]
mod hash_index_tests;

impl EditorTransactionEngine {
    fn metadata_bound_v1(
        &self,
        state: &EngineState,
        additional_history: bool,
        storage: FreshHashStorageV1,
        active_capacity: usize,
    ) -> Result<u64, EditCommandError> {
        let mut m = RetainedBoundBuilderV1::new(self.retention_domain.model());
        m.layout(std::mem::size_of::<Self>(), std::mem::align_of::<Self>())?;
        m.layout(
            std::mem::size_of::<EngineState>(),
            std::mem::align_of::<EngineState>(),
        )?;
        m.array::<(u64, HistoryContextId)>(4096)?;
        m.add(48)?;
        m.add(storage.allocation::<(HistoryContextId, HistoryStore)>()?)?;
        m.btree::<HistoryContextId, u64>(state.history_generations.len().saturating_add(1))?;
        m.array::<ActiveTransaction>(active_capacity)?;
        for store in state.histories.values() {
            m.array::<super::TransactionRecord>(store.backing_capacity_v1())?;
        }
        if additional_history {
            m.array::<super::TransactionRecord>(
                state
                    .history_capacity
                    .checked_add(1)
                    .ok_or(RetentionContractError::Overflow)?,
            )?;
        }
        Ok(m.bytes().get())
    }
    pub(super) fn ensure_history_retention_v1(
        &self,
        history: HistoryContextId,
    ) -> Result<(), EditCommandError> {
        let mut state = self.lock_state();
        if state.histories.contains_key(&history) {
            return Ok(());
        }
        let request = state
            .histories
            .len()
            .checked_add(1)
            .ok_or(RetentionContractError::Overflow)?
            .max(
                state
                    .histories
                    .len()
                    .checked_mul(2)
                    .ok_or(RetentionContractError::Overflow)?,
            );
        let storage = FreshHashStorageV1::for_request::<(HistoryContextId, HistoryStore)>(request)?;
        let desired = self.metadata_bound_v1(&state, true, storage, state.active.capacity())?;
        let old_bytes = state
            .history_storage_v1
            .allocation::<(HistoryContextId, HistoryStore)>()?;
        self.retention_metadata.reserve(
            desired,
            desired
                .checked_add(old_bytes)
                .ok_or(RetentionContractError::Overflow)?,
            std::iter::empty(),
            0,
            0,
        )?;
        let mut replacement = HashMap::with_capacity(request);
        let store = HistoryStore::with_retention_capacity_v1(state.history_capacity)?;
        let mut old = std::mem::take(&mut state.histories);
        replacement.extend(old.drain());
        replacement.insert(history, store);
        state.histories = replacement;
        state.history_storage_v1 = storage;
        drop(state);
        drop(old); // the old backing allocation remains charged until here
        self.retention_metadata.finish_replay(desired)?;
        Ok(())
    }
    pub(super) fn ensure_active_retention_slot_v1(&self) -> Result<(), EditCommandError> {
        let mut state = self.lock_state();
        if state.active.len() < state.active.capacity() {
            return Ok(());
        }
        let capacity = state
            .active
            .len()
            .checked_add(1)
            .ok_or(RetentionContractError::Overflow)?;
        let desired = self.metadata_bound_v1(&state, false, state.history_storage_v1, capacity)?;
        let old = state
            .active
            .capacity()
            .checked_mul(std::mem::size_of::<ActiveTransaction>())
            .ok_or(RetentionContractError::Overflow)? as u64;
        self.retention_metadata.reserve(
            desired,
            desired
                .checked_add(old)
                .ok_or(RetentionContractError::Overflow)?,
            std::iter::empty(),
            0,
            0,
        )?;
        let additional = capacity.saturating_sub(state.active.len());
        state
            .active
            .try_reserve_exact(additional)
            .map_err(|_| RetentionContractError::Allocation)?;
        if state.active.capacity() != capacity {
            return Err(RetentionContractError::Provider {
                reason: "active Vec capacity differs from supported model",
            }
            .into());
        }
        drop(state);
        self.retention_metadata.finish_replay(desired)?;
        Ok(())
    }
    pub fn pin_history_record_v1(
        &self,
        history: HistoryContextId,
        transaction: TransactionId,
    ) -> Result<crate::core::editing::engine::HistoryRetentionPinV1, EditCommandError> {
        let state = self.lock_state();
        state
            .histories
            .get(&history)
            .and_then(|store| store.retention_owner_v1(transaction))
            .ok_or(RetentionContractError::Stale)?
            .record_pin()
            .map_err(Into::into)
    }
    pub fn history_undo_barrier_v1(
        &self,
        history: HistoryContextId,
        transaction: TransactionId,
    ) -> Result<crate::core::editing::engine::HistoryRetentionPinV1, EditCommandError> {
        let state = self.lock_state();
        state
            .histories
            .get(&history)
            .and_then(|store| store.retention_owner_v1(transaction))
            .ok_or(RetentionContractError::Stale)?
            .undo_barrier()
            .map_err(Into::into)
    }
}

impl ActiveTransaction {
    fn measure_v1(&self, meter: &mut RetainedBoundBuilderV1) -> Result<(), EditCommandError> {
        use zircon_runtime_interface::retained_memory::RetainedOwnerV1;
        meter.layout(
            std::mem::size_of::<super::TransactionRecord>(),
            std::mem::align_of::<super::TransactionRecord>(),
        )?;
        meter.owned(&self.label)?;
        meter.array::<CommandBox>(self.commands.capacity())?;
        for command in &self.commands {
            meter.owned(command.as_ref())?;
        }
        meter.btree::<DocumentId, ()>(self.participants.len())?;
        meter.owned(&self.selection_before)?;
        meter.add(self.selection_ceiling_v1)?;
        Ok(())
    }
}
impl EditorTransactionEngine {
    // The busy reservation prevents edits while all user-defined providers run outside state/ledger locks.
    pub(super) fn incoming_retention_limits_v1(
        &self,
        active: &ActiveTransaction,
    ) -> Result<crate::core::editing::engine::CommandRetentionLimitsV1, EditCommandError> {
        let ancestors = std::mem::take(&mut self.lock_state().active);
        let result = (|| {
            let mut meter = RetainedBoundBuilderV1::with_limit(
                active.retention.model(),
                RetainedBytesV1::new(active.retention.budget().record_bytes),
            );
            let mut count = 0usize;
            for frame in ancestors.iter().chain(std::iter::once(active)) {
                frame.measure_v1(&mut meter)?;
                count = count
                    .checked_add(frame.commands.len())
                    .ok_or(RetentionContractError::Overflow)?;
            }
            // The identical prospective backing bound is used by reserve_active.
            meter.array::<CommandBox>(
                count
                    .checked_add(2)
                    .and_then(|n| n.checked_mul(4))
                    .ok_or(RetentionContractError::Overflow)?,
            )?;
            // Prior commands already reserved the current selection lifetime
            // ceiling; Navigation does not mutate that selection.
            let selection = active.selection_ceiling_v1;
            meter.add(selection)?;
            let base = meter.bytes().get();
            let lease = active.retention.charge_for_payload_v1(0)?;
            let retained = active
                .retention
                .budget()
                .record_bytes
                .checked_sub(
                    base.checked_add(lease)
                        .ok_or(RetentionContractError::Overflow)?,
                )
                .ok_or(RetentionContractError::Budget {
                    budget: "remaining command record",
                    requested: base,
                    limit: active.retention.budget().record_bytes,
                })?;
            let scratch = self
                .lock_state()
                .history_capacity
                .checked_add(1)
                .and_then(|n| n.checked_mul(std::mem::size_of::<super::TransactionRecord>()))
                .ok_or(RetentionContractError::Overflow)? as u64;
            let fixed_peak = base
                .checked_mul(2)
                .and_then(|n| n.checked_add(scratch))
                .and_then(|n| n.checked_add(lease))
                .ok_or(RetentionContractError::Overflow)?;
            let headroom = active.retention.materialization_headroom_v1()?;
            let materialization =
                headroom
                    .checked_sub(fixed_peak)
                    .ok_or(RetentionContractError::Budget {
                        budget: "remaining command materialization",
                        requested: fixed_peak,
                        limit: headroom,
                    })?;
            Ok(crate::core::editing::engine::CommandRetentionLimitsV1 {
                retained_bytes: retained,
                materialization_bytes: materialization,
                selection_bytes: selection,
            })
        })();
        self.lock_state().active = ancestors;
        result
    }

    pub(super) fn reserve_active_retention_v1(
        &self,
        active: &ActiveTransaction,
        incoming: Option<&crate::core::editing::engine::CommandRetentionPlanV1>,
        additional_metadata: u64,
    ) -> Result<(), EditCommandError> {
        let ancestors = std::mem::take(&mut self.lock_state().active);
        let result = (|| {
            let limit = active.retention.budget().record_bytes;
            let mut m = RetainedBoundBuilderV1::with_limit(
                active.retention.model(),
                RetainedBytesV1::new(limit),
            );
            let mut count = 0usize;
            for frame in ancestors.iter().chain(std::iter::once(active)) {
                frame.measure_v1(&mut m)?;
                count = count
                    .checked_add(frame.commands.len())
                    .ok_or(RetentionContractError::Overflow)?;
            }
            let mut extra_peak = 0u64;
            if let Some(plan) = incoming {
                m.add(plan.retained_bytes)?;
                m.add(plan.selection_bytes)?;
                extra_peak = plan.peak_bytes;
                count = count
                    .checked_add(1)
                    .ok_or(RetentionContractError::Overflow)?;
            }
            // Exact Vec growth plus nested merge can coexist; bound both source and destination backing.
            m.array::<CommandBox>(
                count
                    .checked_add(1)
                    .and_then(|v| v.checked_mul(4))
                    .ok_or(RetentionContractError::Overflow)?,
            )?;
            m.add(additional_metadata)?;
            let retained = m.bytes().get().max(active.retention.payload_bytes());
            let scratch = {
                let state = self.lock_state();
                state
                    .history_capacity
                    .checked_add(1)
                    .and_then(|n| n.checked_mul(std::mem::size_of::<super::TransactionRecord>()))
                    .ok_or(RetentionContractError::Overflow)? as u64
            };
            let peak = retained
                .checked_mul(2)
                .and_then(|v| v.checked_add(extra_peak))
                .and_then(|v| v.checked_add(scratch))
                .ok_or(RetentionContractError::Overflow)?;
            let state = self.lock_state();
            let store = state
                .histories
                .get(&active.history)
                .ok_or(RetentionContractError::Stale)?;
            store.reserve_retention_v1(
                &active.retention,
                retained,
                peak.max(active.retention.reserved_peak_v1()),
            )
        })();
        self.lock_state().active = ancestors;
        result
    }
    pub(super) fn restore_active_reservation_v1(
        &self,
        active: &ActiveTransaction,
        checkpoint: crate::core::editing::engine::retention::RetentionReservationCheckpointV1,
    ) {
        let state = self.lock_state();
        state
            .histories
            .get(&active.history)
            .expect("admitted history store")
            .restore_reservation_v1(&active.retention, checkpoint);
    }
}

impl EditorTransactionEngine {
    pub(crate) fn reserve_preview_retention_v1(
        &self,
        history: HistoryContextId,
        payload: u64,
        preparation_peak: u64,
    ) -> Result<RootRetentionV1, EditCommandError> {
        self.flush_operation_group()?;
        self.start_operation("reserve interactive transform")?;
        let result = (|| {
            self.ensure_history_retention_v1(history)?;
            let mut meter = RetainedBoundBuilderV1::new(self.retention_domain.model());
            meter.layout(
                std::mem::size_of::<ActiveTransaction>(),
                std::mem::align_of::<ActiveTransaction>(),
            )?;
            meter.layout(
                std::mem::size_of::<super::TransactionRecord>(),
                std::mem::align_of::<super::TransactionRecord>(),
            )?;
            meter.btree::<DocumentId, ()>(1)?;
            meter.array::<CommandBox>(8)?;
            meter.array::<u8>(64)?; // longest static gizmo label (including its transient event copy)
            meter.add(payload)?;
            let retained = meter.bytes().get();
            let scratch = self
                .lock_state()
                .history_capacity
                .checked_add(1)
                .and_then(|n| n.checked_mul(std::mem::size_of::<super::TransactionRecord>()))
                .ok_or(RetentionContractError::Overflow)? as u64;
            let peak = retained
                .checked_mul(2)
                .and_then(|n| n.checked_add(preparation_peak))
                .and_then(|n| n.checked_add(scratch))
                .ok_or(RetentionContractError::Overflow)?;
            let owner = RootRetentionV1::new(&self.retention_domain, 0, 0)?;
            owner.bind_history_v1(&self.retention_domain, history)?;
            self.lock_state()
                .histories
                .get(&history)
                .expect("admitted preview history")
                .reserve_retention_v1(&owner, retained, peak)?;
            Ok(owner)
        })();
        self.clear_operation();
        result
    }
}

impl EditorTransactionEngine {
    pub(crate) fn retention_configuration_v1(
        &self,
    ) -> (
        crate::core::editing::engine::retention::RetainedAccountingModelV1,
        crate::core::editing::engine::HistoryRetentionBudgetV1,
    ) {
        (
            self.retention_domain.model(),
            self.retention_domain.budget(),
        )
    }
}

impl EditorTransactionEngine {
    pub(super) fn restore_begin_reservation_v1(
        &self,
        history: HistoryContextId,
        owner: &RootRetentionV1,
        checkpoint: crate::core::editing::engine::retention::RetentionReservationCheckpointV1,
    ) {
        let state = self.lock_state();
        state
            .histories
            .get(&history)
            .expect("admitted history storage")
            .restore_reservation_v1(owner, checkpoint);
    }
}
