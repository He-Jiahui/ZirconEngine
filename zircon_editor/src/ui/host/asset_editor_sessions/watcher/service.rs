use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use zircon_runtime::asset::project::ProjectManager;

use super::budget::{UiAssetWatchBudget, UiAssetWatchPollAllowance};
use super::diagnostics::{UiAssetWorkspaceWatchDiagnostics, UiAssetWorkspaceWatchPollReport};
use super::ingress::{UiAssetWatchIngressHandle, UiAssetWatchIngressSnapshot};
use super::path_identity::asset_id_for_watched_path;
use crate::ui::host::EditorError;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::host::asset_editor_sessions) struct UiAssetWatchReconcileCursor {
    pub(in crate::ui::host::asset_editor_sessions) current_instance_id: Option<ViewInstanceId>,
    pub(in crate::ui::host::asset_editor_sessions) next_item_index: usize,
}

pub(super) enum UiAssetWatchPollStart {
    Ready(UiAssetWorkspaceWatchPollReport),
    Reconcile {
        cursor: UiAssetWatchReconcileCursor,
        allowance: UiAssetWatchPollAllowance,
    },
}

pub(crate) struct UiAssetWorkspaceWatcher {
    asset_roots: Vec<PathBuf>,
    ingress: UiAssetWatchIngressHandle,
    reconcile_cursor: Option<UiAssetWatchReconcileCursor>,
    reconcile_started_at: Option<Instant>,
    budget: UiAssetWatchBudget,
    _watchers: Vec<RecommendedWatcher>,
}

impl UiAssetWorkspaceWatcher {
    pub(crate) fn start(project: &ProjectManager) -> Result<Self, EditorError> {
        let budget = UiAssetWatchBudget::try_new(4_096, 256, Duration::from_millis(2))
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let asset_roots = project.project_asset_roots().to_vec();
        let ingress = UiAssetWatchIngressHandle::new(budget.max_pending_paths);
        let mut watchers = Vec::with_capacity(asset_roots.len());
        for asset_root in &asset_roots {
            let callback_ingress = ingress.clone();
            let mut watcher =
                notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                    let Ok(event) = event else {
                        return;
                    };
                    callback_ingress.record_paths(event.paths);
                })?;
            watcher.watch(asset_root, RecursiveMode::Recursive)?;
            watchers.push(watcher);
        }
        Ok(Self {
            asset_roots,
            ingress,
            reconcile_cursor: None,
            reconcile_started_at: None,
            budget,
            _watchers: watchers,
        })
    }

    pub(super) fn begin_poll(&mut self) -> UiAssetWatchPollStart {
        let observed_at = Instant::now();
        let mut allowance = self.budget.start_poll();
        if self.ingress.take_overflow() {
            self.reconcile_cursor = Some(UiAssetWatchReconcileCursor::default());
            self.reconcile_started_at = Some(observed_at);
        }
        if let Some(cursor) = self.reconcile_cursor.take() {
            return UiAssetWatchPollStart::Reconcile { cursor, allowance };
        }
        UiAssetWatchPollStart::Ready(
            self.poll_paths_with_allowance(BTreeSet::new(), &mut allowance),
        )
    }

    pub(super) fn finish_reconcile(
        &mut self,
        cursor: Option<UiAssetWatchReconcileCursor>,
        mut allowance: UiAssetWatchPollAllowance,
        changed_asset_ids: BTreeSet<String>,
    ) -> UiAssetWorkspaceWatchPollReport {
        self.reconcile_cursor = cursor;
        if self.reconcile_cursor.is_none() {
            self.reconcile_started_at = None;
        }

        let ingress = self.ingress.snapshot(Instant::now());
        if self.reconcile_cursor.is_none() && !ingress.overflow_pending && !allowance.exhausted() {
            return self.poll_paths_with_allowance(changed_asset_ids, &mut allowance);
        }
        self.report(changed_asset_ids, ingress, &allowance, Instant::now())
    }

    fn poll_paths_with_allowance(
        &self,
        mut changed_asset_ids: BTreeSet<String>,
        allowance: &mut UiAssetWatchPollAllowance,
    ) -> UiAssetWorkspaceWatchPollReport {
        let drained = self.ingress.drain_paths(allowance.remaining_items());
        let mut unprocessed = Vec::new();
        let mut iterator = drained.into_iter();
        while let Some(pending) = iterator.next() {
            if !allowance.try_take() {
                unprocessed.push(pending);
                unprocessed.extend(iterator);
                break;
            }
            if let Some(asset_id) = asset_id_for_watched_path(&self.asset_roots, &pending.path) {
                let _ = changed_asset_ids.insert(asset_id);
            }
        }
        self.ingress.restore_paths_front(unprocessed);
        let ingress = self.ingress.snapshot(Instant::now());
        self.report(changed_asset_ids, ingress, allowance, Instant::now())
    }

    fn report(
        &self,
        changed_asset_ids: BTreeSet<String>,
        ingress: UiAssetWatchIngressSnapshot,
        allowance: &UiAssetWatchPollAllowance,
        observed_at: Instant,
    ) -> UiAssetWorkspaceWatchPollReport {
        let reconcile_age = self
            .reconcile_started_at
            .map(|started_at| observed_at.saturating_duration_since(started_at))
            .unwrap_or_default();
        let reconcile_cursor_active = self.reconcile_cursor.is_some() || ingress.overflow_pending;
        let has_pending = ingress.pending_path_count > 0 || reconcile_cursor_active;
        UiAssetWorkspaceWatchPollReport {
            changed_asset_ids: changed_asset_ids.into_iter().collect(),
            diagnostics: UiAssetWorkspaceWatchDiagnostics {
                pending_path_count: ingress.pending_path_count,
                reconcile_cursor_active,
                received_path_count: ingress.received_path_count,
                coalesced_path_count: ingress.coalesced_path_count,
                overflow_count: ingress.overflow_count,
                oldest_pending_age: ingress.oldest_pending_age.max(reconcile_age),
                budget_exhausted: has_pending && allowance.exhausted(),
                refresh_pending_asset_count: 0,
                refresh_active: false,
                refresh_deferred_retry_count: 0,
                refresh_exhausted_retry_count: 0,
                refresh_superseded_count: 0,
            },
        }
    }

    #[cfg(test)]
    pub(super) fn without_notify_for_test(
        asset_roots: Vec<PathBuf>,
        budget: UiAssetWatchBudget,
    ) -> Self {
        Self {
            asset_roots,
            ingress: UiAssetWatchIngressHandle::new(budget.max_pending_paths),
            reconcile_cursor: None,
            reconcile_started_at: None,
            budget,
            _watchers: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(super) fn record_paths_for_test(&self, paths: impl IntoIterator<Item = PathBuf>) {
        self.ingress.record_paths(paths);
    }
}
