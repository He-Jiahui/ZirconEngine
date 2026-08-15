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
            if state.errors.len() == WATCH_ACTIVATION_ERROR_CAPACITY {
                state.errors.remove(0);
                mark_reconciliation_required(&mut state);
            }
            state.errors.push(error);
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

    fn take_work(&self) -> Option<(AssetWatchBatch, Vec<AssetWatchError>)> {
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

#[cfg(test)]
mod tests {
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
            errors: Vec::new(),
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
}
