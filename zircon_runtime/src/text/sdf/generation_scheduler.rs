//! Bounded non-blocking admission for runtime distance-field generation.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::runtime::tasks::TaskPool;

use super::{
    SdfBakeParams, SdfGenerationBatch, SdfGenerationSourceContext, SdfGenerationSourceHandle,
};

const DEFAULT_MAX_GLYPHS_PER_BATCH: usize = 32;
const DEFAULT_SOURCE_BYTE_BUDGET: usize = 64 * 1024 * 1024;
const DEFAULT_COMPLETION_BYTE_BUDGET: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SdfGenerationWorkId {
    generation: u64,
    index: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SdfGenerationSchedulerOptions {
    max_in_flight_batches: usize,
    max_glyphs_per_batch: usize,
    max_in_flight_glyphs: usize,
    source_byte_budget: usize,
    completion_queue_depth: usize,
    completion_byte_budget: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SdfGenerationSchedulerDiagnostics {
    pub(crate) in_flight_batch_count: usize,
    pub(crate) in_flight_glyph_count: usize,
    pub(crate) in_flight_source_byte_count: usize,
    pub(crate) queue_peak: usize,
    pub(crate) completion_backlog_count: usize,
    pub(crate) completion_backlog_byte_count: usize,
    pub(crate) completed_batch_count: u64,
    pub(crate) completed_glyph_count: u64,
    pub(crate) failed_glyph_count: u64,
    pub(crate) cancelled_batch_count: u64,
    pub(crate) rejected_batch_count: u64,
    pub(crate) completion_backpressured_count: u64,
    pub(crate) worker_panic_count: u64,
    pub(crate) oldest_in_flight_age_frames: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SdfGenerationSubmitError {
    EmptyBatch(SdfGenerationWorkId),
    BatchTooLarge(SdfGenerationWorkId),
    Duplicate(SdfGenerationWorkId),
    QueueFull(SdfGenerationWorkId),
    GlyphBudgetFull(SdfGenerationWorkId),
    SourceByteBudgetFull(SdfGenerationWorkId),
    Shutdown(SdfGenerationWorkId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SdfGenerationInactiveWorkOutcome {
    Retryable,
    WorkerPanic,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SdfGenerationCompletion {
    pub(crate) id: SdfGenerationWorkId,
    pub(crate) age_frames: u64,
    pub(crate) batch: SdfGenerationBatch,
    byte_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SdfGenerationCompletionDrainBudget {
    max_items: usize,
    max_bytes: usize,
}

struct InFlightBatch {
    submitted_frame: u64,
    glyph_count: usize,
    source: SdfGenerationSourceHandle,
    cancelled: bool,
}

#[derive(Clone, Copy)]
struct InFlightSource {
    reference_count: usize,
    byte_count: usize,
}

#[derive(Default)]
struct SchedulerState {
    active_ids: HashSet<SdfGenerationWorkId>,
    worker_panic_ids: HashSet<SdfGenerationWorkId>,
    in_flight: HashMap<SdfGenerationWorkId, InFlightBatch>,
    in_flight_glyph_count: usize,
    in_flight_sources: HashMap<SdfGenerationSourceHandle, InFlightSource>,
    in_flight_source_byte_count: usize,
    completions: VecDeque<SdfGenerationCompletion>,
    completion_byte_count: usize,
    diagnostics: SdfGenerationSchedulerDiagnostics,
}

pub(crate) struct SdfGenerationScheduler {
    pool: TaskPool,
    options: SdfGenerationSchedulerOptions,
    state: Arc<Mutex<SchedulerState>>,
    shutdown: Arc<AtomicBool>,
}

impl SdfGenerationWorkId {
    pub(crate) const fn new(generation: u64, index: u64) -> Self {
        Self { generation, index }
    }
}

impl SdfGenerationSchedulerOptions {
    pub(crate) fn new(max_in_flight_batches: usize) -> Self {
        let max_in_flight_batches = max_in_flight_batches.max(1);
        Self {
            max_in_flight_batches,
            max_glyphs_per_batch: DEFAULT_MAX_GLYPHS_PER_BATCH,
            max_in_flight_glyphs: max_in_flight_batches
                .saturating_mul(DEFAULT_MAX_GLYPHS_PER_BATCH),
            source_byte_budget: DEFAULT_SOURCE_BYTE_BUDGET,
            completion_queue_depth: max_in_flight_batches,
            completion_byte_budget: DEFAULT_COMPLETION_BYTE_BUDGET,
        }
    }

    pub(crate) fn with_max_glyphs_per_batch(mut self, value: usize) -> Self {
        self.max_glyphs_per_batch = value.max(1);
        self
    }

    pub(crate) fn with_max_in_flight_glyphs(mut self, value: usize) -> Self {
        self.max_in_flight_glyphs = value.max(1);
        self
    }

    pub(crate) fn with_source_byte_budget(mut self, value: usize) -> Self {
        self.source_byte_budget = value.max(1);
        self
    }

    pub(crate) fn with_completion_queue_depth(mut self, value: usize) -> Self {
        self.completion_queue_depth = value.max(1);
        self
    }

    pub(crate) fn with_completion_byte_budget(mut self, value: usize) -> Self {
        self.completion_byte_budget = value.max(1);
        self
    }
}

impl SdfGenerationCompletionDrainBudget {
    pub(crate) const fn new(max_items: usize, max_bytes: usize) -> Self {
        Self {
            max_items,
            max_bytes,
        }
    }
}

impl SdfGenerationScheduler {
    pub(crate) fn new(pool: TaskPool, options: SdfGenerationSchedulerOptions) -> Self {
        Self {
            pool,
            options,
            state: Arc::new(Mutex::new(SchedulerState::default())),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) const fn max_glyphs_per_batch(&self) -> usize {
        self.options.max_glyphs_per_batch
    }

    pub(crate) fn try_submit(
        &self,
        id: SdfGenerationWorkId,
        frame_index: u64,
        source: Arc<SdfGenerationSourceContext>,
        params: SdfBakeParams,
        glyph_ids: Vec<u16>,
    ) -> Result<(), SdfGenerationSubmitError> {
        if self.shutdown.load(Ordering::Acquire) {
            return Err(SdfGenerationSubmitError::Shutdown(id));
        }
        if glyph_ids.is_empty() {
            return Err(SdfGenerationSubmitError::EmptyBatch(id));
        }
        if glyph_ids.len() > self.options.max_glyphs_per_batch {
            return Err(SdfGenerationSubmitError::BatchTooLarge(id));
        }

        let source_handle = source.handle();
        let source_byte_count = source.report().source_byte_len;
        {
            let mut state = lock_state(&self.state);
            let rejection = if state.active_ids.contains(&id) {
                Some(SdfGenerationSubmitError::Duplicate(id))
            } else if state.in_flight.len() >= self.options.max_in_flight_batches {
                Some(SdfGenerationSubmitError::QueueFull(id))
            } else if state.in_flight_glyph_count.saturating_add(glyph_ids.len())
                > self.options.max_in_flight_glyphs
            {
                Some(SdfGenerationSubmitError::GlyphBudgetFull(id))
            } else {
                let added_source_bytes = state
                    .in_flight_sources
                    .contains_key(&source_handle)
                    .then_some(0)
                    .unwrap_or(source_byte_count);
                (state
                    .in_flight_source_byte_count
                    .saturating_add(added_source_bytes)
                    > self.options.source_byte_budget)
                    .then_some(SdfGenerationSubmitError::SourceByteBudgetFull(id))
            };
            if let Some(error) = rejection {
                state.diagnostics.rejected_batch_count =
                    state.diagnostics.rejected_batch_count.saturating_add(1);
                return Err(error);
            }

            state.worker_panic_ids.remove(&id);
            state.active_ids.insert(id);
            state.in_flight_glyph_count =
                state.in_flight_glyph_count.saturating_add(glyph_ids.len());
            admit_source(&mut state, source_handle, source_byte_count);
            state.in_flight.insert(
                id,
                InFlightBatch {
                    submitted_frame: frame_index,
                    glyph_count: glyph_ids.len(),
                    source: source_handle,
                    cancelled: false,
                },
            );
            let in_flight_count = state.in_flight.len();
            state.diagnostics.queue_peak = state.diagnostics.queue_peak.max(in_flight_count);
        }

        let state = Arc::clone(&self.state);
        let shutdown = Arc::clone(&self.shutdown);
        let options = self.options;
        self.pool.spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                source.generate_batch(params, &glyph_ids)
            }));
            publish_worker_result(&state, &shutdown, options, id, result);
        });
        Ok(())
    }

    pub(crate) fn cancel(&self, id: SdfGenerationWorkId) -> bool {
        let mut state = lock_state(&self.state);
        state.worker_panic_ids.remove(&id);
        let Some(batch) = state.in_flight.get_mut(&id) else {
            return false;
        };
        if batch.cancelled {
            return false;
        }
        batch.cancelled = true;
        true
    }

    pub(crate) fn cancel_all(&self) -> usize {
        let mut state = lock_state(&self.state);
        state.worker_panic_ids.clear();
        let mut cancelled_count = 0;
        for batch in state.in_flight.values_mut() {
            if !batch.cancelled {
                batch.cancelled = true;
                cancelled_count += 1;
            }
        }
        cancelled_count
    }

    pub(crate) fn drain_completed(
        &self,
        frame_index: u64,
        budget: SdfGenerationCompletionDrainBudget,
    ) -> Vec<SdfGenerationCompletion> {
        let mut state = lock_state(&self.state);
        let mut drained = Vec::new();
        let mut drained_bytes = 0_usize;
        while drained.len() < budget.max_items {
            let Some(next) = state.completions.front() else {
                break;
            };
            if !drained.is_empty()
                && drained_bytes.saturating_add(next.byte_count) > budget.max_bytes
            {
                break;
            }
            let Some(mut completion) = state.completions.pop_front() else {
                break;
            };
            completion.age_frames = frame_index.saturating_sub(completion.age_frames);
            drained_bytes = drained_bytes.saturating_add(completion.byte_count);
            state.completion_byte_count = state
                .completion_byte_count
                .saturating_sub(completion.byte_count);
            state.active_ids.remove(&completion.id);
            drained.push(completion);
        }
        drained
    }

    pub(crate) fn diagnostics(&self, frame_index: u64) -> SdfGenerationSchedulerDiagnostics {
        let state = lock_state(&self.state);
        let mut diagnostics = state.diagnostics;
        diagnostics.in_flight_batch_count = state.in_flight.len();
        diagnostics.in_flight_glyph_count = state.in_flight_glyph_count;
        diagnostics.in_flight_source_byte_count = state.in_flight_source_byte_count;
        diagnostics.completion_backlog_count = state.completions.len();
        diagnostics.completion_backlog_byte_count = state.completion_byte_count;
        diagnostics.oldest_in_flight_age_frames = state
            .in_flight
            .values()
            .map(|batch| frame_index.saturating_sub(batch.submitted_frame))
            .max()
            .unwrap_or(0);
        diagnostics
    }

    pub(crate) fn take_inactive_work_outcomes(
        &self,
        ids: impl IntoIterator<Item = SdfGenerationWorkId>,
    ) -> Vec<(SdfGenerationWorkId, SdfGenerationInactiveWorkOutcome)> {
        let mut state = lock_state(&self.state);
        ids.into_iter()
            .filter_map(|id| {
                (!state.active_ids.contains(&id)).then(|| {
                    let outcome = state
                        .worker_panic_ids
                        .remove(&id)
                        .then_some(SdfGenerationInactiveWorkOutcome::WorkerPanic)
                        .unwrap_or(SdfGenerationInactiveWorkOutcome::Retryable);
                    (id, outcome)
                })
            })
            .collect()
    }
}

impl Drop for SdfGenerationScheduler {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.cancel_all();
    }
}

fn publish_worker_result(
    state: &Mutex<SchedulerState>,
    shutdown: &AtomicBool,
    options: SdfGenerationSchedulerOptions,
    id: SdfGenerationWorkId,
    result: Result<SdfGenerationBatch, Box<dyn std::any::Any + Send>>,
) {
    let mut state = lock_state(state);
    let Some(in_flight) = state.in_flight.remove(&id) else {
        state.active_ids.remove(&id);
        return;
    };
    state.in_flight_glyph_count = state
        .in_flight_glyph_count
        .saturating_sub(in_flight.glyph_count);
    release_source(&mut state, in_flight.source);

    if shutdown.load(Ordering::Acquire) || in_flight.cancelled {
        state.active_ids.remove(&id);
        state.diagnostics.cancelled_batch_count =
            state.diagnostics.cancelled_batch_count.saturating_add(1);
        return;
    }
    let Ok(batch) = result else {
        state.active_ids.remove(&id);
        state.worker_panic_ids.insert(id);
        state.diagnostics.worker_panic_count =
            state.diagnostics.worker_panic_count.saturating_add(1);
        return;
    };
    let byte_count = batch_byte_count(&batch);
    if state.completions.len() >= options.completion_queue_depth
        || state.completion_byte_count.saturating_add(byte_count) > options.completion_byte_budget
    {
        state.active_ids.remove(&id);
        state.diagnostics.completion_backpressured_count = state
            .diagnostics
            .completion_backpressured_count
            .saturating_add(1);
        return;
    }
    state.diagnostics.completed_batch_count =
        state.diagnostics.completed_batch_count.saturating_add(1);
    state.diagnostics.completed_glyph_count = state
        .diagnostics
        .completed_glyph_count
        .saturating_add(batch.report.generated_glyph_count as u64);
    state.diagnostics.failed_glyph_count = state
        .diagnostics
        .failed_glyph_count
        .saturating_add(batch.report.failed_glyph_count as u64);
    state.completion_byte_count = state.completion_byte_count.saturating_add(byte_count);
    state.completions.push_back(SdfGenerationCompletion {
        id,
        age_frames: in_flight.submitted_frame,
        batch,
        byte_count,
    });
}

fn admit_source(state: &mut SchedulerState, handle: SdfGenerationSourceHandle, byte_count: usize) {
    if let Some(source) = state.in_flight_sources.get_mut(&handle) {
        source.reference_count = source.reference_count.saturating_add(1);
        return;
    }
    state.in_flight_sources.insert(
        handle,
        InFlightSource {
            reference_count: 1,
            byte_count,
        },
    );
    state.in_flight_source_byte_count =
        state.in_flight_source_byte_count.saturating_add(byte_count);
}

fn release_source(state: &mut SchedulerState, handle: SdfGenerationSourceHandle) {
    let Some(mut source) = state.in_flight_sources.get(&handle).copied() else {
        return;
    };
    source.reference_count = source.reference_count.saturating_sub(1);
    if source.reference_count != 0 {
        state.in_flight_sources.insert(handle, source);
        return;
    }
    let byte_count = source.byte_count;
    state.in_flight_sources.remove(&handle);
    state.in_flight_source_byte_count =
        state.in_flight_source_byte_count.saturating_sub(byte_count);
}

fn batch_byte_count(batch: &SdfGenerationBatch) -> usize {
    batch
        .glyphs
        .iter()
        .filter_map(|glyph| glyph.result.as_ref().ok())
        .map(|glyph| glyph.pixels.len())
        .sum()
}

fn lock_state(state: &Mutex<SchedulerState>) -> MutexGuard<'_, SchedulerState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::runtime::tasks::TaskPoolDescriptor;
    use crate::text::VariationCoords;
    use std::sync::Arc;

    fn fixture_source(handle: u64) -> Arc<SdfGenerationSourceContext> {
        let bytes = Arc::<[u8]>::from(
            std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("assets/fonts/FiraSans-Regular.ttf"),
            )
            .expect("Fira Sans fixture"),
        );
        Arc::new(
            SdfGenerationSourceContext::new(
                SdfGenerationSourceHandle::new(handle),
                bytes,
                0,
                Arc::new(VariationCoords::default()),
            )
            .expect("parsed generation source"),
        )
    }

    #[test]
    fn worker_panic_is_reported_as_terminal_inactive_work() {
        let scheduler = SdfGenerationScheduler::new(
            TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1)),
            SdfGenerationSchedulerOptions::new(1),
        );
        let work_id = SdfGenerationWorkId::new(7, 3);
        let source = SdfGenerationSourceHandle::new(11);
        {
            let mut state = lock_state(&scheduler.state);
            state.active_ids.insert(work_id);
            state.in_flight.insert(
                work_id,
                InFlightBatch {
                    submitted_frame: 4,
                    glyph_count: 1,
                    source,
                    cancelled: false,
                },
            );
            state.in_flight_glyph_count = 1;
            admit_source(&mut state, source, 128);
        }

        publish_worker_result(
            scheduler.state.as_ref(),
            scheduler.shutdown.as_ref(),
            scheduler.options,
            work_id,
            Err(Box::new("test worker panic")),
        );

        assert_eq!(
            scheduler.take_inactive_work_outcomes([work_id]),
            vec![(work_id, SdfGenerationInactiveWorkOutcome::WorkerPanic)]
        );
        assert!(scheduler.take_inactive_work_outcomes([work_id]).is_empty());
        assert_eq!(scheduler.diagnostics(5).worker_panic_count, 1);
    }

    #[test]
    fn cancelling_an_inactive_work_id_discards_its_stale_panic_outcome() {
        let scheduler = SdfGenerationScheduler::new(
            TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1)),
            SdfGenerationSchedulerOptions::new(1),
        );
        let work_id = SdfGenerationWorkId::new(7, 3);
        lock_state(&scheduler.state)
            .worker_panic_ids
            .insert(work_id);

        assert!(!scheduler.cancel(work_id));
        assert_eq!(
            scheduler.take_inactive_work_outcomes([work_id]),
            vec![(work_id, SdfGenerationInactiveWorkOutcome::Retryable)]
        );
    }

    #[test]
    fn cancelling_all_work_discards_stale_panic_outcomes() {
        let scheduler = SdfGenerationScheduler::new(
            TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1)),
            SdfGenerationSchedulerOptions::new(1),
        );
        let work_id = SdfGenerationWorkId::new(7, 3);
        lock_state(&scheduler.state)
            .worker_panic_ids
            .insert(work_id);

        assert_eq!(scheduler.cancel_all(), 0);
        assert_eq!(
            scheduler.take_inactive_work_outcomes([work_id]),
            vec![(work_id, SdfGenerationInactiveWorkOutcome::Retryable)]
        );
    }

    #[test]
    fn admitting_a_reused_work_id_discards_its_stale_panic_outcome() {
        let scheduler = SdfGenerationScheduler::new(
            TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1)),
            SdfGenerationSchedulerOptions::new(1),
        );
        let work_id = SdfGenerationWorkId::new(7, 3);
        lock_state(&scheduler.state)
            .worker_panic_ids
            .insert(work_id);

        scheduler
            .try_submit(
                work_id,
                1,
                fixture_source(12),
                SdfBakeParams::default(),
                vec![1],
            )
            .expect("reused work id must be admitted");

        assert!(!lock_state(&scheduler.state)
            .worker_panic_ids
            .contains(&work_id));
    }
}
