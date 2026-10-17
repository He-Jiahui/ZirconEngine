//! Versioned requested-allocation accounting; allocator overhead and RSS are outside this model.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use super::{EditCommand, EditCommandError, EditContext};
pub use zircon_runtime_interface::retained_memory::{
    verified_std_model_v1, RetainedAccountingModelV1, RetainedBoundBuilderV1, RetainedBoundV1,
    RetainedBytesV1, RetainedOwnerV1, RetentionContractError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetentionPhaseV1 {
    Apply,
    Undo,
    Redo,
    Merge,
    Rollback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryRetentionBudgetV1 {
    pub record_bytes: u64,
    pub history_bytes: u64,
    pub committed_bytes: u64,
    pub staging_bytes: u64,
}
impl Default for HistoryRetentionBudgetV1 {
    fn default() -> Self {
        // Engineering defaults, not numeric thresholds from the original Editor02 plan.
        Self {
            record_bytes: 64 * 1024 * 1024,
            history_bytes: 64 * 1024 * 1024,
            committed_bytes: 256 * 1024 * 1024,
            staging_bytes: 64 * 1024 * 1024,
        }
    }
}
impl HistoryRetentionBudgetV1 {
    fn validate(self) -> Result<Self, RetentionContractError> {
        if self.record_bytes == 0
            || self.record_bytes > self.history_bytes
            || self.history_bytes > self.committed_bytes
            || self.staging_bytes == 0
            || self
                .committed_bytes
                .checked_add(self.staging_bytes)
                .is_none()
        {
            return Err(RetentionContractError::Provider {
                reason: "invalid history retention budgets",
            });
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HistoryRetentionSnapshotV1 {
    pub committed_bytes: u64,
    pub retiring_bytes: u64,
    pub staging_bytes: u64,
    pub promised_bytes: u64,
    pub metadata_bytes: u64,
}
#[derive(Clone)]
pub struct HistoryRetentionDomainV1(Arc<DomainInner>);
struct DomainInner {
    budget: HistoryRetentionBudgetV1,
    model: RetainedAccountingModelV1,
    state: Mutex<Ledger>,
}
struct Ledger {
    next: u64,
    entries: Vec<Entry>,
    metadata: u64,
    retiring_metadata: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Staging,
    Committed,
    Retiring,
}
#[derive(Clone, Copy)]
struct Entry {
    id: u64,
    phase: Phase,
    retained: u64,
    desired: u64,
    staging: u64,
    promised: u64,
    pledged_to: Option<u64>,
    awaiting: usize,
    ready: bool,
    prefix: usize,
    redo: usize,
    history: Option<super::HistoryContextId>,
}

fn add(a: u64, b: u64) -> Result<u64, RetentionContractError> {
    a.checked_add(b).ok_or(RetentionContractError::Overflow)
}
fn array_bytes<T>(capacity: usize) -> Result<u64, RetentionContractError> {
    let bytes = capacity
        .checked_mul(std::mem::size_of::<T>())
        .ok_or(RetentionContractError::Overflow)?;
    if bytes > isize::MAX as usize {
        return Err(RetentionContractError::Overflow);
    }
    u64::try_from(bytes).map_err(|_| RetentionContractError::Overflow)
}
fn arc_bytes<T>() -> Result<u64, RetentionContractError> {
    use zircon_runtime_interface::retained_memory::align_up_v1;
    let a = std::mem::align_of::<T>();
    let bytes = align_up_v1(16, a)?
        .checked_add(std::mem::size_of::<T>())
        .ok_or(RetentionContractError::Overflow)?;
    u64::try_from(align_up_v1(bytes, a.max(8))?).map_err(|_| RetentionContractError::Overflow)
}
impl Ledger {
    fn snapshot(&self) -> Result<HistoryRetentionSnapshotV1, RetentionContractError> {
        let mut s = HistoryRetentionSnapshotV1 {
            committed_bytes: self.metadata,
            retiring_bytes: self.retiring_metadata,
            metadata_bytes: add(self.metadata, self.retiring_metadata)?,
            ..Default::default()
        };
        for e in &self.entries {
            match e.phase {
                Phase::Committed => s.committed_bytes = add(s.committed_bytes, e.retained)?,
                Phase::Retiring => s.retiring_bytes = add(s.retiring_bytes, e.retained)?,
                Phase::Staging => {}
            }
            s.staging_bytes = add(s.staging_bytes, e.staging)?;
            s.promised_bytes = add(s.promised_bytes, e.promised)?;
        }
        Ok(s)
    }
    fn index(&self, id: u64) -> Option<usize> {
        self.entries.iter().position(|e| e.id == id)
    }
    fn check(&self, budget: HistoryRetentionBudgetV1) -> Result<(), RetentionContractError> {
        let s = self.snapshot()?;
        let projected = add(add(s.committed_bytes, s.retiring_bytes)?, s.promised_bytes)?;
        if projected > budget.committed_bytes {
            return Err(RetentionContractError::Budget {
                budget: "global committed",
                requested: projected,
                limit: budget.committed_bytes,
            });
        }
        if s.staging_bytes > budget.staging_bytes {
            return Err(RetentionContractError::Budget {
                budget: "global staging",
                requested: s.staging_bytes,
                limit: budget.staging_bytes,
            });
        }
        Ok(())
    }
    fn promote_ready(&mut self, id: u64) {
        if let Some(i) = self.index(id) {
            let e = &mut self.entries[i];
            if e.ready && e.awaiting == 0 {
                e.phase = Phase::Committed;
                e.staging = 0;
                e.promised = 0;
            }
        }
    }
}
impl HistoryRetentionDomainV1 {
    pub fn try_new(budget: HistoryRetentionBudgetV1) -> Result<Self, RetentionContractError> {
        let budget = budget.validate()?;
        let model = verified_std_model_v1()?;
        let metadata = arc_bytes::<DomainInner>()?;
        let state = Ledger {
            next: 1,
            entries: Vec::new(),
            metadata,
            retiring_metadata: 0,
        };
        state.check(budget)?;
        Ok(Self(Arc::new(DomainInner {
            budget,
            model,
            state: Mutex::new(state),
        })))
    }
    pub fn shared_default() -> Result<Self, RetentionContractError> {
        static DEFAULT: OnceLock<Result<HistoryRetentionDomainV1, RetentionContractError>> =
            OnceLock::new();
        DEFAULT
            .get_or_init(|| Self::try_new(HistoryRetentionBudgetV1::default()))
            .clone()
    }
    fn lock(&self) -> MutexGuard<'_, Ledger> {
        self.0.state.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub fn budget(&self) -> HistoryRetentionBudgetV1 {
        self.0.budget
    }
    pub fn model(&self) -> RetainedAccountingModelV1 {
        self.0.model
    }
    pub fn snapshot(&self) -> Result<HistoryRetentionSnapshotV1, RetentionContractError> {
        self.lock().snapshot()
    }

    /// Only fixed bookkeeping is moved while locked. Old backing storage is dropped unlocked.
    fn owner(&self, retained: u64, peak: u64) -> Result<RootRetentionV1, RetentionContractError> {
        let overhead = arc_bytes::<LeaseOwner>()?;
        let retained = add(retained, overhead)?;
        let peak = add(peak, overhead)?;
        let mut old_backing = None;
        let id;
        {
            let mut l = self.lock();
            id = l.next;
            let next = id.checked_add(1).ok_or(RetentionContractError::Overflow)?;
            let entry = Entry {
                id,
                phase: Phase::Staging,
                retained,
                desired: retained,
                staging: peak,
                promised: retained,
                pledged_to: None,
                awaiting: 0,
                ready: false,
                prefix: 0,
                redo: 0,
                history: None,
            };
            // Reserve old+new metadata peak before allocation; never infer hash storage from capacity.
            if l.entries.len() == l.entries.capacity() {
                let capacity = l
                    .entries
                    .capacity()
                    .max(4)
                    .checked_mul(2)
                    .ok_or(RetentionContractError::Overflow)?;
                let bytes = array_bytes::<Entry>(capacity)?;
                let old_bytes = array_bytes::<Entry>(l.entries.capacity())?;
                let s = l.snapshot()?;
                let projected = add(
                    add(add(s.committed_bytes, s.retiring_bytes)?, s.promised_bytes)?,
                    add(bytes, retained)?,
                )?;
                if projected > self.0.budget.committed_bytes
                    || add(s.staging_bytes, add(bytes, peak)?)? > self.0.budget.staging_bytes
                {
                    return Err(RetentionContractError::Budget {
                        budget: "ledger metadata growth",
                        requested: projected,
                        limit: self.0.budget.committed_bytes,
                    });
                }
                let mut replacement = Vec::new();
                replacement
                    .try_reserve_exact(capacity)
                    .map_err(|_| RetentionContractError::Allocation)?;
                let actual = array_bytes::<Entry>(replacement.capacity())?;
                // Global's admitted Vec model returns the requested capacity.
                if actual != bytes {
                    return Err(RetentionContractError::Provider {
                        reason: "Vec metadata capacity differs from admitted model",
                    });
                }
                replacement.append(&mut l.entries);
                old_backing = Some((std::mem::replace(&mut l.entries, replacement), old_bytes));
                l.metadata = l.metadata - old_bytes + bytes;
                l.retiring_metadata += old_bytes;
            }
            l.entries.push(entry);
            if let Err(error) = l.check(self.0.budget) {
                l.entries.pop();
                drop(l);
                if let Some((old, bytes)) = old_backing {
                    drop(old);
                    self.lock().retiring_metadata -= bytes;
                }
                return Err(error);
            }
            l.next = next;
        }
        if let Some((old, bytes)) = old_backing {
            drop(old);
            self.lock().retiring_metadata -= bytes;
        }
        Ok(RootRetentionV1(Arc::new(LeaseOwner {
            domain: self.clone(),
            id,
            pins: AtomicUsize::new(0),
            barriers: AtomicUsize::new(0),
            record_limited: true,
        })))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RetentionReservationCheckpointV1(Entry);
impl RetentionReservationCheckpointV1 {
    pub(crate) fn retirement_plan(self) -> (usize, usize) {
        (self.0.prefix, self.0.redo)
    }
}

struct LeaseOwner {
    domain: HistoryRetentionDomainV1,
    id: u64,
    pins: AtomicUsize,
    barriers: AtomicUsize,
    record_limited: bool,
}
impl Drop for LeaseOwner {
    fn drop(&mut self) {
        let mut l = self.domain.lock();
        let Some(i) = l.index(self.id) else {
            return;
        };
        let e = l.entries.swap_remove(i);
        for old in &mut l.entries {
            if old.pledged_to == Some(self.id) {
                old.pledged_to = None;
            }
        }
        if let Some(target) = e.pledged_to {
            if let Some(i) = l.index(target) {
                let t = &mut l.entries[i];
                t.awaiting = t.awaiting.saturating_sub(1);
                // Released old credit becomes an earmarked promise in the same critical section.
                t.promised = t.retained.min(t.promised.saturating_add(e.retained));
            }
            l.promote_ready(target);
        }
    }
}
#[derive(Clone)]
pub(crate) struct RootRetentionV1(Arc<LeaseOwner>);
impl std::fmt::Debug for RootRetentionV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RootRetentionV1").field(&self.0.id).finish()
    }
}
impl RootRetentionV1 {
    pub(crate) fn new(
        domain: &HistoryRetentionDomainV1,
        retained: u64,
        peak: u64,
    ) -> Result<Self, RetentionContractError> {
        domain.owner(retained, peak)
    }
    pub(crate) fn metadata(
        domain: &HistoryRetentionDomainV1,
        bytes: u64,
    ) -> Result<Self, RetentionContractError> {
        let mut owner = domain.owner(bytes, bytes)?;
        Arc::get_mut(&mut owner.0)
            .expect("new metadata owner is unique")
            .record_limited = false;
        owner.ready();
        Ok(owner)
    }
    pub(crate) fn bind_history_v1(
        &self,
        domain: &HistoryRetentionDomainV1,
        history: super::HistoryContextId,
    ) -> Result<(), RetentionContractError> {
        if !Arc::ptr_eq(&self.0.domain.0, &domain.0) {
            return Err(RetentionContractError::Stale);
        }
        let mut l = self.0.domain.lock();
        let i = l.index(self.id()).ok_or(RetentionContractError::Stale)?;
        if l.entries[i].history.is_some_and(|bound| bound != history) {
            return Err(RetentionContractError::Stale);
        }
        l.entries[i].history = Some(history);
        Ok(())
    }
    pub(crate) fn id(&self) -> u64 {
        self.0.id
    }
    pub(crate) fn charge_for_payload_v1(&self, bytes: u64) -> Result<u64, RetentionContractError> {
        add(bytes, arc_bytes::<LeaseOwner>()?)
    }
    pub(crate) fn retained(&self) -> u64 {
        let l = self.0.domain.lock();
        l.index(self.id()).map_or(0, |i| l.entries[i].retained)
    }
    pub(crate) fn protected(&self) -> bool {
        self.0.pins.load(Ordering::Acquire) != 0 || self.0.barriers.load(Ordering::Acquire) != 0
    }
    pub(crate) fn model(&self) -> RetainedAccountingModelV1 {
        self.0.domain.model()
    }
    pub(crate) fn budget(&self) -> HistoryRetentionBudgetV1 {
        self.0.domain.budget()
    }
    pub(crate) fn reserve<I>(
        &self,
        retained: u64,
        peak: u64,
        old: I,
        prefix: usize,
        redo: usize,
    ) -> Result<(), RetentionContractError>
    where
        I: Iterator<Item = u64> + Clone,
    {
        let mut l = self.0.domain.lock();
        let i = l.index(self.id()).ok_or(RetentionContractError::Stale)?;
        let original = l.entries[i];
        if original.phase == Phase::Retiring || original.pledged_to.is_some() {
            return Err(RetentionContractError::Stale);
        }
        let mut credit = 0;
        let mut count = 0usize;
        for id in old.clone() {
            let entry = l
                .index(id)
                .map(|i| l.entries[i])
                .ok_or(RetentionContractError::Stale)?;
            if id == self.id()
                || entry.phase != Phase::Committed
                || entry.pledged_to.is_some_and(|p| p != self.id())
            {
                return Err(RetentionContractError::Stale);
            }
            credit = add(credit, entry.retained)?;
            count = count
                .checked_add(1)
                .ok_or(RetentionContractError::Overflow)?;
        }
        let overhead = arc_bytes::<LeaseOwner>()?;
        let retained = add(retained, overhead)?;
        let peak = add(peak.max(retained.saturating_sub(overhead)), overhead)?;
        if self.0.record_limited && retained > self.budget().record_bytes {
            return Err(RetentionContractError::Budget {
                budget: "one history record",
                requested: retained,
                limit: self.budget().record_bytes,
            });
        }
        let staging = if original.phase == Phase::Staging {
            peak
        } else {
            peak.saturating_sub(original.retained)
        };
        l.entries[i].desired = retained;
        l.entries[i].staging = staging;
        l.entries[i].promised = retained.saturating_sub(add(
            credit,
            if original.phase == Phase::Staging {
                0
            } else {
                original.retained
            },
        )?);
        // Retained is the desired total only for new owners; existing replay keeps its old baseline until success.
        if original.phase == Phase::Staging {
            l.entries[i].retained = retained;
        }
        if let Err(error) = l.check(self.0.domain.0.budget) {
            l.entries[i] = original;
            return Err(error);
        }
        for e in &mut l.entries {
            if e.pledged_to == Some(self.id()) {
                e.pledged_to = None;
            }
        }
        for id in old {
            if let Some(i) = l.index(id) {
                l.entries[i].pledged_to = Some(self.id());
            }
        }
        let i = l.index(self.id()).ok_or(RetentionContractError::Stale)?;
        l.entries[i].awaiting = count;
        l.entries[i].prefix = prefix;
        l.entries[i].redo = redo;
        Ok(())
    }

    /// Reserves a borrowed selection's temporary capture before Arc/default
    /// allocation. Existing promises, target-only credits and retained charges
    /// stay intact; the next whole-frame quote subsumes this peak.
    pub(crate) fn reserve_capture_v1(&self, bytes: u64) -> Result<(), RetentionContractError> {
        let mut ledger = self.0.domain.lock();
        let i = ledger
            .index(self.id())
            .ok_or(RetentionContractError::Stale)?;
        let original = ledger.entries[i];
        if original.phase == Phase::Retiring || original.pledged_to.is_some() {
            return Err(RetentionContractError::Stale);
        }
        let peak = add(original.desired, bytes)?;
        let staging = if original.phase == Phase::Staging {
            peak
        } else {
            peak.saturating_sub(original.retained)
        };
        ledger.entries[i].staging = original.staging.max(staging);
        if let Err(error) = ledger.check(self.budget()) {
            ledger.entries[i] = original;
            return Err(error);
        }
        Ok(())
    }

    /// The caller keeps this owner alive until the returned snapshot is dropped
    /// or transferred to a record carrying this same owner.
    pub(crate) fn capture_selection_v1(
        &self,
        context: &dyn EditContext,
    ) -> Result<super::SelectionSnapshot, EditCommandError> {
        let bound =
            context.selection_retention_bound_v1(self.model(), self.budget().record_bytes)?;
        self.reserve_capture_v1(bound)?;
        let snapshot = context.selection_snapshot();
        let mut meter =
            RetainedBoundBuilderV1::with_limit(self.model(), RetainedBytesV1::new(bound));
        meter.owned(&snapshot)?;
        Ok(snapshot)
    }

    pub(crate) fn checkpoint_v1(&self) -> RetentionReservationCheckpointV1 {
        let l = self.0.domain.lock();
        RetentionReservationCheckpointV1(
            l.entries[l.index(self.id()).expect("live retention owner")],
        )
    }
    pub(crate) fn restore_reservation_v1(
        &self,
        checkpoint: RetentionReservationCheckpointV1,
        old: impl Iterator<Item = u64>,
    ) {
        let mut l = self.0.domain.lock();
        for e in &mut l.entries {
            if e.pledged_to == Some(self.id()) {
                e.pledged_to = None;
            }
        }
        for id in old {
            if let Some(i) = l.index(id) {
                l.entries[i].pledged_to = Some(self.id());
            }
        }
        let i = l.index(self.id()).expect("live retention owner");
        l.entries[i] = checkpoint.0;
    }
    pub(crate) fn retire(&self) {
        let mut ledger = self.0.domain.lock();
        if let Some(i) = ledger.index(self.id()) {
            ledger.entries[i].phase = Phase::Retiring;
        }
    }
    pub(crate) fn materialization_headroom_v1(&self) -> Result<u64, RetentionContractError> {
        let ledger = self.0.domain.lock();
        let entry = ledger.entries[ledger
            .index(self.id())
            .ok_or(RetentionContractError::Stale)?];
        let others = ledger
            .snapshot()?
            .staging_bytes
            .checked_sub(entry.staging)
            .ok_or(RetentionContractError::Overflow)?;
        let remaining = self.budget().staging_bytes.checked_sub(others).ok_or(
            RetentionContractError::Budget {
                budget: "other history staging owners",
                requested: others,
                limit: self.budget().staging_bytes,
            },
        )?;
        match entry.phase {
            Phase::Staging => Ok(remaining),
            Phase::Committed => add(remaining, entry.retained),
            Phase::Retiring => Err(RetentionContractError::Stale),
        }
    }
    pub(crate) fn reserved_peak_v1(&self) -> u64 {
        let l = self.0.domain.lock();
        l.index(self.id())
            .map_or(0, |i| l.entries[i].staging.max(l.entries[i].desired))
    }
    pub(crate) fn retirement_plan(&self) -> (usize, usize) {
        let l = self.0.domain.lock();
        l.index(self.id())
            .map_or((0, 0), |i| (l.entries[i].prefix, l.entries[i].redo))
    }
    pub(crate) fn ready(&self) {
        let mut l = self.0.domain.lock();
        for e in &mut l.entries {
            if e.pledged_to == Some(self.id()) {
                e.phase = Phase::Retiring;
            }
        }
        if let Some(i) = l.index(self.id()) {
            l.entries[i].ready = true;
        }
        l.promote_ready(self.id());
    }
    pub(crate) fn finish_replay(&self, retained: u64) -> Result<(), RetentionContractError> {
        let retained = add(retained, arc_bytes::<LeaseOwner>()?)?;
        let mut l = self.0.domain.lock();
        let i = l.index(self.id()).ok_or(RetentionContractError::Stale)?;
        if retained > l.entries[i].desired {
            return Err(RetentionContractError::Provider {
                reason: "owner exceeded its prepared retained bound",
            });
        }
        l.entries[i].retained = retained;
        l.entries[i].desired = retained;
        l.entries[i].staging = 0;
        l.entries[i].promised = 0;
        Ok(())
    }
    pub(crate) fn payload_bytes(&self) -> u64 {
        self.retained()
            .saturating_sub(arc_bytes::<LeaseOwner>().unwrap_or(u64::MAX))
    }
    fn pin(&self, barrier: bool) -> Result<HistoryRetentionPinV1, RetentionContractError> {
        let l = self.0.domain.lock();
        let e = l.index(self.id()).ok_or(RetentionContractError::Stale)?;
        if l.entries[e].pledged_to.is_some() {
            return Err(RetentionContractError::Provider {
                reason: "record retirement already reserved",
            });
        }
        let counter = if barrier {
            &self.0.barriers
        } else {
            &self.0.pins
        };
        counter
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
            .map_err(|_| RetentionContractError::Overflow)?;
        Ok(HistoryRetentionPinV1 {
            owner: self.clone(),
            barrier,
        })
    }
    pub(crate) fn is_barrier(&self) -> bool {
        self.0.barriers.load(Ordering::Acquire) != 0
    }
    pub(crate) fn record_pin(&self) -> Result<HistoryRetentionPinV1, RetentionContractError> {
        self.pin(false)
    }
    pub(crate) fn undo_barrier(&self) -> Result<HistoryRetentionPinV1, RetentionContractError> {
        self.pin(true)
    }
}
/// A real owner-bound pin, distinct from a Copy save token.
pub struct HistoryRetentionPinV1 {
    owner: RootRetentionV1,
    barrier: bool,
}
impl Drop for HistoryRetentionPinV1 {
    fn drop(&mut self) {
        let counter = if self.barrier {
            &self.owner.0.barriers
        } else {
            &self.owner.0.pins
        };
        counter.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Residual command allowance after the engine's real frames, selection,
/// prospective command backing, record-finalization scratch and lease layout.
/// A command's 2 * retained_bytes + peak_bytes must fit materialization_bytes.
#[derive(Clone, Copy, Debug)]
pub struct CommandRetentionLimitsV1 {
    pub retained_bytes: u64,
    pub materialization_bytes: u64,
    pub selection_bytes: u64,
}

pub struct CommandRetentionPlanV1 {
    identity: usize,
    pub phase: RetentionPhaseV1,
    pub retained_bytes: u64,
    pub peak_bytes: u64,
    pub selection_bytes: u64,
    pub owner_revision: Option<u64>,
    pub create_quote_v1:
        Option<zircon_runtime::core::framework::retained_memory::RetainedCreateQuoteV1>,
    /// Finite retained/peak allowance transferred to a versioned protocol receiver.
    pub transfer_budget_v1: Option<(u64, u64)>,
}
impl CommandRetentionPlanV1 {
    pub fn for_owner<T: RetainedOwnerV1 + ?Sized>(
        owner: &T,
        context: &dyn EditContext,
        phase: RetentionPhaseV1,
        model: RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<Self, EditCommandError> {
        let mut m = RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(limit));
        m.owned(owner)?;
        let bytes = m.bytes().get();
        let selection = context.selection_retention_bound_v1(model, limit)?;
        Ok(Self {
            identity: std::ptr::from_ref(owner).cast::<()>() as usize,
            phase,
            retained_bytes: bytes,
            peak_bytes: bytes,
            selection_bytes: selection,
            owner_revision: None,
            create_quote_v1: None,
            transfer_budget_v1: None,
        })
    }
    pub fn add_selection(&mut self, bytes: u64) -> Result<(), EditCommandError> {
        self.selection_bytes = add(self.selection_bytes, bytes)?;
        Ok(())
    }
    pub fn add_retained(&mut self, bytes: u64) -> Result<(), EditCommandError> {
        self.retained_bytes = add(self.retained_bytes, bytes)?;
        self.peak_bytes = add(self.peak_bytes, bytes)?;
        Ok(())
    }
    pub fn add_peak(&mut self, bytes: u64) -> Result<(), EditCommandError> {
        self.peak_bytes = add(self.peak_bytes, bytes)?;
        Ok(())
    }
}
/// A non-cloneable one-preparation capability. It owns no opaque unmetered payload.
pub struct RetentionMaterializationV1 {
    plan: CommandRetentionPlanV1,
    model: RetainedAccountingModelV1,
    consumed: bool,
}
impl RetentionMaterializationV1 {
    pub(crate) fn new(plan: CommandRetentionPlanV1, model: RetainedAccountingModelV1) -> Self {
        Self {
            plan,
            model,
            consumed: false,
        }
    }
    pub fn model(&self) -> RetainedAccountingModelV1 {
        self.model
    }
    pub fn plan(&self) -> &CommandRetentionPlanV1 {
        &self.plan
    }
    pub fn consume<T: ?Sized>(
        &mut self,
        owner: &T,
        phase: RetentionPhaseV1,
    ) -> Result<(), EditCommandError> {
        if self.consumed
            || self.plan.identity != std::ptr::from_ref(owner).cast::<()>() as usize
            || self.plan.phase != phase
        {
            return Err(RetentionContractError::Stale.into());
        }
        self.consumed = true;
        Ok(())
    }
    pub(crate) fn finish_v1(&self) -> Result<(), EditCommandError> {
        if !self.consumed {
            return Err(RetentionContractError::Provider {
                reason: "command did not consume its preparation permit",
            }
            .into());
        }
        Ok(())
    }
    pub fn check_prepared<T: RetainedOwnerV1 + ?Sized>(
        &self,
        owner: &T,
    ) -> Result<(), EditCommandError> {
        let mut m = RetainedBoundBuilderV1::with_limit(
            self.model,
            RetainedBytesV1::new(self.plan.peak_bytes),
        );
        m.owned(owner)?;
        Ok(())
    }
}

pub(crate) fn prepare_under_owner_v1(
    command: &mut dyn EditCommand,
    context: &mut dyn EditContext,
    owner: &RootRetentionV1,
    phase: RetentionPhaseV1,
) -> Result<(), super::CommandExecutionError> {
    let plan = command
        .retention_plan_v1(context, phase, owner.model(), owner.budget().record_bytes)
        .map_err(super::CommandExecutionError::unchanged)?;
    if plan.peak_bytes > owner.reserved_peak_v1() {
        return Err(super::CommandExecutionError::unchanged(
            RetentionContractError::Provider {
                reason: "compensation exceeded its reserved lifetime peak",
            }
            .into(),
        ));
    }
    let mut permit = RetentionMaterializationV1::new(plan, owner.model());
    command.prepare_retention_v1(context, phase, &mut permit)?;
    permit
        .finish_v1()
        .map_err(super::CommandExecutionError::unchanged)
}
