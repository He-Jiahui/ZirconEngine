use std::sync::Arc;

use crate::asset::watch::{
    AssetChange, AssetChangeKind, AssetWatchBatch, AssetWatchBatchDiagnostics, AssetWatchError,
};
use crate::asset::AssetUri;

use super::project_asset_manager::{
    ProjectWatcherActivation, ProjectWatcherActivationState, ProjectWatcherLifecycle,
};
use super::ProjectAssetManager;

const WATCH_ACTIVATION_ENTRY_CAPACITY: usize = 4_096;
const WATCH_ACTIVATION_BYTE_CAPACITY: usize = 4 * 1024 * 1024;
const WATCH_ACTIVATION_ERROR_CAPACITY: usize = 64;

impl ProjectWatcherActivation {
    pub(super) fn enqueue_batch(
        self: &Arc<Self>,
        manager: &ProjectAssetManager,
        batch: AssetWatchBatch,
    ) {
        let should_schedule = {
            let mut state = self.lock_state();
            if state.lifecycle == ProjectWatcherLifecycle::Retired {
                return;
            }
            merge_batch(&mut state, batch);
            should_schedule_worker(&mut state)
        };
        if should_schedule {
            self.spawn_worker(manager.clone());
        }
    }

    pub(super) fn enqueue_error(
        self: &Arc<Self>,
        manager: &ProjectAssetManager,
        error: AssetWatchError,
    ) {
        let should_schedule = {
            let mut state = self.lock_state();
            if state.lifecycle == ProjectWatcherLifecycle::Retired {
                return;
            }
            if push_bounded_error(&mut state.errors, error, WATCH_ACTIVATION_ERROR_CAPACITY) {
                mark_reconciliation_required(&mut state);
            }
            should_schedule_worker(&mut state)
        };
        if should_schedule {
            self.spawn_worker(manager.clone());
        }
    }

    pub(super) fn activate_dispatch(self: &Arc<Self>, manager: &ProjectAssetManager) {
        let should_schedule = {
            let mut state = self.lock_state();
            if state.lifecycle != ProjectWatcherLifecycle::Draining {
                return;
            }
            state.lifecycle = ProjectWatcherLifecycle::Active;
            should_schedule_worker(&mut state)
        };
        if should_schedule {
            self.spawn_worker(manager.clone());
        }
    }

    fn spawn_worker(self: &Arc<Self>, manager: ProjectAssetManager) {
        let activation = self.clone();
        manager.worker_task_pool().clone().spawn(move || {
            activation.run_worker(manager);
        });
    }

    fn run_worker(&self, manager: ProjectAssetManager) {
        loop {
            let Some((batch, errors)) = self.take_work() else {
                return;
            };
            for error in errors {
                manager.broadcast_watch_error(error);
            }
            if !batch.is_empty() {
                manager.process_watch_batch_in_generation(batch);
            }
        }
    }

    fn take_work(&self) -> Option<(AssetWatchBatch, std::collections::VecDeque<AssetWatchError>)> {
        let mut state = self.lock_state();
        if state.lifecycle == ProjectWatcherLifecycle::Retired {
            state.worker_scheduled = false;
            return None;
        }
        if state.changes.is_empty() && state.errors.is_empty() && !state.requires_reconciliation {
            state.worker_scheduled = false;
            return None;
        }
        let work = (
            AssetWatchBatch {
                changes: std::mem::take(&mut state.changes),
                requires_reconciliation: std::mem::take(&mut state.requires_reconciliation),
                diagnostics: std::mem::take(&mut state.diagnostics),
            },
            std::mem::take(&mut state.errors),
        );
        state.coalescible_change_indices.clear();
        state.queued_change_bytes = 0;
        Some(work)
    }
}

fn should_schedule_worker(state: &mut ProjectWatcherActivationState) -> bool {
    let has_work =
        !state.changes.is_empty() || !state.errors.is_empty() || state.requires_reconciliation;
    if state.lifecycle == ProjectWatcherLifecycle::Active && has_work && !state.worker_scheduled {
        state.worker_scheduled = true;
        true
    } else {
        false
    }
}

fn merge_batch(state: &mut ProjectWatcherActivationState, batch: AssetWatchBatch) {
    merge_diagnostics(&mut state.diagnostics, batch.diagnostics);
    if state.requires_reconciliation || batch.requires_reconciliation {
        state.changes.clear();
        state.coalescible_change_indices.clear();
        state.queued_change_bytes = 0;
        state.requires_reconciliation = true;
        return;
    }
    for change in batch.changes {
        let current_bytes = state.queued_change_bytes;
        state.queued_change_bytes = apply_change(
            &mut state.changes,
            &mut state.coalescible_change_indices,
            change,
            current_bytes,
        );
        if state.changes.len() > WATCH_ACTIVATION_ENTRY_CAPACITY
            || state.queued_change_bytes > WATCH_ACTIVATION_BYTE_CAPACITY
        {
            mark_reconciliation_required(state);
            return;
        }
    }
}

fn apply_change(
    changes: &mut Vec<AssetChange>,
    coalescible_change_indices: &mut std::collections::HashMap<AssetUri, usize>,
    change: AssetChange,
    current_bytes: usize,
) -> usize {
    if matches!(
        change.kind,
        AssetChangeKind::Added | AssetChangeKind::Modified
    ) {
        if let Some(index) = coalescible_change_indices.get(&change.uri).copied() {
            let previous = &mut changes[index];
            if previous.kind != AssetChangeKind::Added {
                let previous_bytes = approximate_change_bytes(previous);
                let next_bytes = approximate_change_bytes(&change);
                *previous = change;
                return current_bytes
                    .saturating_sub(previous_bytes)
                    .saturating_add(next_bytes);
            }
            return current_bytes;
        }
        coalescible_change_indices.insert(change.uri.clone(), changes.len());
        let next_bytes = approximate_change_bytes(&change)
            .saturating_add(approximate_coalescible_index_bytes(&change.uri));
        changes.push(change);
        return current_bytes.saturating_add(next_bytes);
    }

    let mut next_bytes = current_bytes;
    if coalescible_change_indices.remove(&change.uri).is_some() {
        next_bytes = next_bytes.saturating_sub(approximate_coalescible_index_bytes(&change.uri));
    }
    if let Some(previous_uri) = change.previous_uri.as_ref() {
        if previous_uri != &change.uri && coalescible_change_indices.remove(previous_uri).is_some()
        {
            next_bytes =
                next_bytes.saturating_sub(approximate_coalescible_index_bytes(previous_uri));
        }
    }
    let change_bytes = approximate_change_bytes(&change);
    changes.push(change);
    next_bytes.saturating_add(change_bytes)
}

fn mark_reconciliation_required(state: &mut ProjectWatcherActivationState) {
    state.changes.clear();
    state.coalescible_change_indices.clear();
    state.queued_change_bytes = 0;
    state.requires_reconciliation = true;
    state.diagnostics.pending_overflow_count =
        state.diagnostics.pending_overflow_count.saturating_add(1);
}

fn merge_diagnostics(current: &mut AssetWatchBatchDiagnostics, next: AssetWatchBatchDiagnostics) {
    current.raw_event_count = current.raw_event_count.saturating_add(next.raw_event_count);
    current.coalesced_event_count = current
        .coalesced_event_count
        .saturating_add(next.coalesced_event_count);
    current.ingress_overflow_count = current
        .ingress_overflow_count
        .saturating_add(next.ingress_overflow_count);
    current.pending_overflow_count = current
        .pending_overflow_count
        .saturating_add(next.pending_overflow_count);
    current.approximate_bytes = current
        .approximate_bytes
        .saturating_add(next.approximate_bytes);
    current.oldest_age = current.oldest_age.max(next.oldest_age);
}

fn approximate_change_bytes(change: &AssetChange) -> usize {
    std::mem::size_of::<AssetChange>()
        + locator_bytes(&change.uri)
        + change
            .previous_uri
            .as_ref()
            .map(locator_bytes)
            .unwrap_or_default()
}

fn approximate_coalescible_index_bytes(uri: &AssetUri) -> usize {
    std::mem::size_of::<(AssetUri, usize)>() + locator_bytes(uri) + 16
}

fn locator_bytes(uri: &AssetUri) -> usize {
    uri.path().len() + uri.label().map(str::len).unwrap_or_default() + 12
}

fn push_bounded_error<T>(
    errors: &mut std::collections::VecDeque<T>,
    error: T,
    capacity: usize,
) -> bool {
    debug_assert!(capacity > 0);
    let evicted = if errors.len() >= capacity {
        errors.pop_front().is_some()
    } else {
        false
    };
    errors.push_back(error);
    evicted
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::sync::Mutex;
    use std::time::Instant;

    use super::*;

    fn change(index: usize) -> AssetChange {
        AssetChange::new(
            AssetChangeKind::Modified,
            AssetUri::parse(&format!("res://data/watch-{index}.json")).unwrap(),
            None,
        )
    }

    fn state() -> ProjectWatcherActivationState {
        ProjectWatcherActivationState {
            lifecycle: ProjectWatcherLifecycle::Pending,
            changes: Vec::new(),
            coalescible_change_indices: Default::default(),
            queued_change_bytes: 0,
            requires_reconciliation: false,
            diagnostics: AssetWatchBatchDiagnostics::default(),
            errors: Default::default(),
            worker_scheduled: false,
        }
    }

    #[test]
    fn activation_coalesces_modifications_without_crossing_remove_edges() {
        let uri = AssetUri::parse("res://data/watch.json").unwrap();
        let mut state = state();
        merge_batch(
            &mut state,
            AssetWatchBatch {
                changes: vec![
                    AssetChange::new(AssetChangeKind::Modified, uri.clone(), None),
                    AssetChange::new(AssetChangeKind::Modified, uri.clone(), None),
                    AssetChange::new(AssetChangeKind::Removed, uri.clone(), None),
                    AssetChange::new(AssetChangeKind::Added, uri, None),
                ],
                ..AssetWatchBatch::default()
            },
        );

        assert_eq!(state.changes.len(), 3);
        assert_eq!(state.changes[0].kind, AssetChangeKind::Modified);
        assert_eq!(state.changes[1].kind, AssetChangeKind::Removed);
        assert_eq!(state.changes[2].kind, AssetChangeKind::Added);
    }

    #[test]
    fn activation_worker_admission_is_singleflight() {
        let mut state = state();
        state.lifecycle = ProjectWatcherLifecycle::Active;
        state.changes.push(change(0));

        assert!(should_schedule_worker(&mut state));
        assert!(!should_schedule_worker(&mut state));
    }

    #[test]
    fn activation_entry_overflow_discards_partial_queue_and_marks_dirty() {
        let mut state = state();
        merge_batch(
            &mut state,
            AssetWatchBatch {
                changes: (0..=WATCH_ACTIVATION_ENTRY_CAPACITY).map(change).collect(),
                ..AssetWatchBatch::default()
            },
        );

        assert!(state.requires_reconciliation);
        assert!(state.changes.is_empty());
        assert_eq!(state.queued_change_bytes, 0);
        assert_eq!(state.diagnostics.pending_overflow_count, 1);
    }

    #[test]
    fn activation_error_overflow_discards_oldest_and_preserves_fifo_order() {
        let manager = ProjectAssetManager::default();
        let activation = Arc::new(ProjectWatcherActivation {
            state: Mutex::new(state()),
        });
        for index in 0..(WATCH_ACTIVATION_ERROR_CAPACITY + 2) {
            activation.enqueue_error(
                &manager,
                AssetWatchError::from_message(
                    std::path::PathBuf::from("project-assets"),
                    format!("watch-error-{index}"),
                ),
            );
        }

        let (batch, errors) = activation
            .take_work()
            .expect("watch errors should be queued");
        assert!(batch.requires_reconciliation);
        assert_eq!(errors.len(), WATCH_ACTIVATION_ERROR_CAPACITY);
        assert_eq!(errors.front().unwrap().message, "watch-error-2");
        assert_eq!(
            errors.back().unwrap().message,
            format!("watch-error-{}", WATCH_ACTIVATION_ERROR_CAPACITY + 1)
        );
    }

    #[test]
    #[ignore = "managed release performance evidence"]
    fn watch_error_tail_queue_release_benchmark_evidence() {
        const ITEMS: usize = 200_000;
        const SAMPLE_PAIRS: usize = 21;

        let mut legacy_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            let mut measure_legacy = || {
                let started = Instant::now();
                let mut queue = Vec::with_capacity(WATCH_ACTIVATION_ERROR_CAPACITY);
                for item in 0..ITEMS {
                    if queue.len() == WATCH_ACTIVATION_ERROR_CAPACITY {
                        black_box(queue.remove(0));
                    }
                    queue.push(item);
                }
                black_box(queue);
                legacy_samples_ns.push(started.elapsed().as_nanos());
            };
            let mut measure_optimized = || {
                let started = Instant::now();
                let mut queue =
                    std::collections::VecDeque::with_capacity(WATCH_ACTIVATION_ERROR_CAPACITY);
                for item in 0..ITEMS {
                    black_box(push_bounded_error(
                        &mut queue,
                        item,
                        WATCH_ACTIVATION_ERROR_CAPACITY,
                    ));
                }
                black_box(queue);
                optimized_samples_ns.push(started.elapsed().as_nanos());
            };
            if sample_index % 2 == 0 {
                measure_legacy();
                measure_optimized();
            } else {
                measure_optimized();
                measure_legacy();
            }
        }

        let legacy_p95_ns = nearest_rank_percentile(&legacy_samples_ns, 95);
        let optimized_p95_ns = nearest_rank_percentile(&optimized_samples_ns, 95);
        let overflow_count = ITEMS - WATCH_ACTIVATION_ERROR_CAPACITY;
        let legacy_moves = overflow_count * (WATCH_ACTIVATION_ERROR_CAPACITY - 1);
        println!(
            "WATCH_ERROR_TAIL_QUEUE_BENCH_V1 items={ITEMS} capacity={} sample_pairs={SAMPLE_PAIRS} \
             overflow_count={overflow_count} legacy_moves={legacy_moves} optimized_moves=0 \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} \
             optimized_ns={}",
            WATCH_ACTIVATION_ERROR_CAPACITY,
            join_nanosecond_samples(&legacy_samples_ns),
            join_nanosecond_samples(&optimized_samples_ns),
        );
        assert!(
            optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
            "optimized P95 {optimized_p95_ns}ns must be at most 75% of legacy P95 {legacy_p95_ns}ns"
        );
    }

    fn join_nanosecond_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
        assert!(!samples.is_empty());
        assert!((1..=100).contains(&percentile));
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let index = (ordered.len() * percentile).div_ceil(100) - 1;
        ordered[index]
    }
}
