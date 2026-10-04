//! Worker-owned raster execution, batching, and completion publication.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::core::framework::channel::ChannelSender;
use crate::text::raster::{GlyphBitmapError, SwashRasterError, SwashRasterizer};
use crossbeam_channel::SendTimeoutError;

use super::super::completion_queue::CompletionByteBudget;
use super::{
    request_channel, TextRasterWorkId, TextRasterWorkItem, TextRasterWorkResult,
    TextRasterWorkerPool, TextRasterWorkerPoolDiagnostics, TextRasterWorkerPoolOptions,
    TextRasterWorkerWorkState, TEXT_RASTER_WORKER_MAX_SCALER_BATCH_SIZE,
};

const TEXT_RASTER_WORKER_COMPLETION_SEND_TIMEOUT: Duration = Duration::from_millis(1);

pub(super) fn process_worker_batch(
    rasterizer: &mut SwashRasterizer,
    completion_tx: &ChannelSender<TextRasterWorkResult>,
    work_state: &Mutex<TextRasterWorkerWorkState>,
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
    completion_byte_budget: &CompletionByteBudget,
    work_batch: Vec<TextRasterWorkItem>,
) {
    let mut active_work = Vec::with_capacity(work_batch.len());
    for work in work_batch {
        if finish_cancelled_work(work_state, diagnostics, work.id) {
            continue;
        }
        if begin_worker_work(work_state, diagnostics, work.id) {
            active_work.push(work);
        }
    }

    while !active_work.is_empty() {
        let first_work = active_work.remove(0);
        let mut compatible_work = vec![first_work];
        let mut index = 0;
        while index < active_work.len() {
            if raster_work_items_share_scaler(&compatible_work[0], &active_work[index]) {
                compatible_work.push(active_work.swap_remove(index));
            } else {
                index += 1;
            }
        }

        let requests = compatible_work
            .iter()
            .map(|work| work.request.clone())
            .collect::<Vec<_>>();
        rasterizer.rasterize_batch(
            compatible_work[0].font_data.as_ref(),
            &requests,
            |index, result| {
                let work = &compatible_work[index];
                publish_completion(
                    completion_tx,
                    work_state,
                    diagnostics,
                    completion_byte_budget,
                    TextRasterWorkResult {
                        id: work.id,
                        face_epoch: work.face_epoch,
                        result,
                    },
                );
            },
        );
    }
}

fn raster_work_items_share_scaler(first: &TextRasterWorkItem, other: &TextRasterWorkItem) -> bool {
    Arc::ptr_eq(&first.font_data, &other.font_data)
        && first
            .request
            .shares_scaler_configuration_with(&other.request)
}

fn publish_completion(
    completion_tx: &ChannelSender<TextRasterWorkResult>,
    work_state: &Mutex<TextRasterWorkerWorkState>,
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
    completion_byte_budget: &CompletionByteBudget,
    mut result: TextRasterWorkResult,
) {
    let result_id = result.id;
    let mut result_bytes = result.byte_count();
    if !completion_byte_budget.try_reserve(result_bytes) {
        record_completion_budget_rejected(diagnostics, result_bytes);
        result.result = Err(SwashRasterError::InvalidGlyphBitmap(
            GlyphBitmapError::DataLengthMismatch {
                expected: 0,
                actual: result_bytes,
            },
        ));
        result_bytes = 0;
        if !completion_byte_budget.try_reserve(result_bytes) {
            let _ = finish_worker_work(work_state, diagnostics, result_id, true);
            return;
        }
    }
    let failed = result.result.is_err();
    record_completion_backlog(diagnostics, result_bytes);

    loop {
        if finish_cancelled_work(work_state, diagnostics, result_id) {
            completion_byte_budget.release(result_bytes);
            release_completion_backlog(diagnostics, result_bytes);
            return;
        }
        match completion_tx.send_timeout(result, TEXT_RASTER_WORKER_COMPLETION_SEND_TIMEOUT) {
            Ok(()) => {
                let _ = finish_worker_work(work_state, diagnostics, result_id, failed);
                return;
            }
            Err(SendTimeoutError::Timeout(next_result)) => {
                record_completion_backpressured(diagnostics);
                result = next_result;
            }
            Err(SendTimeoutError::Disconnected(next_result)) => {
                completion_byte_budget.release(result_bytes);
                release_completion_backlog(diagnostics, result_bytes);
                let _ = finish_worker_work(work_state, diagnostics, next_result.id, failed);
                return;
            }
        }
    }
}

fn begin_worker_work(
    work_state: &Mutex<TextRasterWorkerWorkState>,
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
    work_id: TextRasterWorkId,
) -> bool {
    let (in_flight_count, running_count) = {
        let mut work_state = lock_worker_work_state(work_state);
        if !work_state.in_flight.contains(&work_id) {
            return false;
        }
        work_state.running.insert(work_id);
        (work_state.in_flight.len(), work_state.running.len())
    };
    let mut diagnostics = lock_worker_diagnostics(diagnostics);
    diagnostics.in_flight = in_flight_count;
    diagnostics.running = running_count;
    diagnostics.queued = in_flight_count.saturating_sub(running_count);
    diagnostics.queued_input_bytes = in_flight_count
        .saturating_sub(running_count)
        .saturating_mul(std::mem::size_of::<TextRasterWorkItem>());
    true
}

fn is_worker_work_cancelled(
    work_state: &Mutex<TextRasterWorkerWorkState>,
    work_id: TextRasterWorkId,
) -> bool {
    lock_worker_work_state(work_state)
        .cancelled
        .contains(&work_id)
}

pub(super) fn record_completion_backlog(
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
    result_bytes: usize,
) {
    let mut diagnostics = lock_worker_diagnostics(diagnostics);
    diagnostics.completion_backlog = diagnostics.completion_backlog.saturating_add(1);
    diagnostics.completion_backlog_bytes = diagnostics
        .completion_backlog_bytes
        .saturating_add(result_bytes);
}

pub(super) fn release_completion_backlog(
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
    result_bytes: usize,
) {
    let mut diagnostics = lock_worker_diagnostics(diagnostics);
    diagnostics.completion_backlog = diagnostics.completion_backlog.saturating_sub(1);
    diagnostics.completion_backlog_bytes = diagnostics
        .completion_backlog_bytes
        .saturating_sub(result_bytes);
}

pub(super) fn record_completion_backpressured(
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
) {
    let mut diagnostics = lock_worker_diagnostics(diagnostics);
    diagnostics.completion_backpressured = diagnostics.completion_backpressured.saturating_add(1);
}

pub(super) fn record_completion_budget_rejected(
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
    result_bytes: usize,
) {
    let mut diagnostics = lock_worker_diagnostics(diagnostics);
    diagnostics.completion_budget_rejected =
        diagnostics.completion_budget_rejected.saturating_add(1);
    diagnostics.completion_rejected_bytes = diagnostics
        .completion_rejected_bytes
        .saturating_add(result_bytes as u64);
}

fn finish_cancelled_work(
    work_state: &Mutex<TextRasterWorkerWorkState>,
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
    work_id: TextRasterWorkId,
) -> bool {
    let Some(remaining_work) = (|| {
        let mut work_state = lock_worker_work_state(work_state);
        work_state.cancelled.remove(&work_id).then(|| {
            work_state.in_flight.remove(&work_id);
            work_state.running.remove(&work_id);
            (work_state.in_flight.len(), work_state.running.len())
        })
    })() else {
        return false;
    };
    let mut diagnostics = lock_worker_diagnostics(diagnostics);
    diagnostics.in_flight = remaining_work.0;
    diagnostics.running = remaining_work.1;
    diagnostics.queued = remaining_work.0.saturating_sub(remaining_work.1);
    diagnostics.queued_input_bytes = remaining_work
        .0
        .saturating_sub(remaining_work.1)
        .saturating_mul(std::mem::size_of::<TextRasterWorkItem>());
    diagnostics.cancelled = diagnostics.cancelled.saturating_add(1);
    true
}

fn finish_worker_work(
    work_state: &Mutex<TextRasterWorkerWorkState>,
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
    work_id: TextRasterWorkId,
    failed: bool,
) -> bool {
    let (was_cancelled, remaining_work, running_work) = {
        let mut work_state = lock_worker_work_state(work_state);
        let was_cancelled = work_state.cancelled.remove(&work_id);
        work_state.in_flight.remove(&work_id);
        work_state.running.remove(&work_id);
        (
            was_cancelled,
            work_state.in_flight.len(),
            work_state.running.len(),
        )
    };
    let mut diagnostics = lock_worker_diagnostics(diagnostics);
    diagnostics.in_flight = remaining_work;
    diagnostics.running = running_work;
    diagnostics.queued = remaining_work.saturating_sub(running_work);
    diagnostics.queued_input_bytes = remaining_work
        .saturating_sub(running_work)
        .saturating_mul(std::mem::size_of::<TextRasterWorkItem>());
    if was_cancelled {
        diagnostics.cancelled = diagnostics.cancelled.saturating_add(1);
        return false;
    }
    diagnostics.completed = diagnostics.completed.saturating_add(1);
    if failed {
        diagnostics.failed = diagnostics.failed.saturating_add(1);
    }
    true
}

pub(super) fn lock_worker_work_state(
    work_state: &Mutex<TextRasterWorkerWorkState>,
) -> MutexGuard<'_, TextRasterWorkerWorkState> {
    work_state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn lock_worker_diagnostics(
    diagnostics: &Mutex<TextRasterWorkerPoolDiagnostics>,
) -> MutexGuard<'_, TextRasterWorkerPoolDiagnostics> {
    diagnostics
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
impl TextRasterWorkerPool {
    pub(crate) fn new_without_workers_for_test(options: TextRasterWorkerPoolOptions) -> Self {
        let options = options.normalized();
        let completion_byte_budget = options.completion_byte_budget;
        let (request_tx, request_rx) = request_channel(options.queue_depth);
        let (completion_tx, completion_rx) =
            crossbeam_channel::bounded(options.completion_queue_depth);
        let diagnostics = Arc::new(Mutex::new(TextRasterWorkerPoolDiagnostics::for_options(
            &options,
        )));

        Self {
            options,
            request_tx: Some(request_tx),
            request_rx_guard: Some(request_rx),
            work_state: Arc::new(Mutex::new(TextRasterWorkerWorkState::default())),
            diagnostics,
            completion_tx,
            completion_rx: Some(completion_rx),
            deferred_completion: Mutex::new(None),
            completion_byte_budget: Arc::new(CompletionByteBudget::new(completion_byte_budget)),
            shutdown: Arc::new(AtomicBool::new(false)),
            joins: Vec::new(),
        }
    }

    pub(crate) fn request_channel_guard_is_alive_for_test(&self) -> bool {
        self.request_rx_guard.is_some()
    }

    pub(crate) fn disconnect_request_channel_for_test(&mut self) {
        self.request_tx.take();
        self.request_rx_guard.take();
    }

    pub(crate) fn try_recv_request_for_test(&self) -> Option<TextRasterWorkItem> {
        self.request_rx_guard
            .as_ref()
            .and_then(|request_rx| request_rx.try_recv().ok())
    }

    pub(crate) fn process_next_request_for_test(&self) -> bool {
        let Some(work) = self.try_recv_request_for_test() else {
            return false;
        };
        let mut rasterizer = SwashRasterizer::new();
        process_worker_batch(
            &mut rasterizer,
            &self.completion_tx,
            &self.work_state,
            &self.diagnostics,
            &self.completion_byte_budget,
            vec![work],
        );
        true
    }

    pub(crate) fn process_next_batch_for_test(&self) -> usize {
        let Some(first_work) = self.try_recv_request_for_test() else {
            return 0;
        };
        let mut work_batch = Vec::with_capacity(TEXT_RASTER_WORKER_MAX_SCALER_BATCH_SIZE);
        work_batch.push(first_work);
        while work_batch.len() < TEXT_RASTER_WORKER_MAX_SCALER_BATCH_SIZE {
            let Some(next_work) = self.try_recv_request_for_test() else {
                break;
            };
            work_batch.push(next_work);
        }
        let processed_count = work_batch.len();
        let mut rasterizer = SwashRasterizer::new();
        process_worker_batch(
            &mut rasterizer,
            &self.completion_tx,
            &self.work_state,
            &self.diagnostics,
            &self.completion_byte_budget,
            work_batch,
        );
        processed_count
    }
}
