use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use crate::core::runtime::{TaskCancellationPolicy, TaskDescriptor, TaskId, TaskPoolKind};

use super::contract::{
    RenderArtifactManifestIoDispatchBudget, RenderArtifactManifestIoDispatchError,
    RenderArtifactManifestIoDispatchReport,
};
use super::loader::RenderArtifactManifestLoaderInner;
use super::state::take_task_id;
use super::worker::run_manifest_io_task;

impl RenderArtifactManifestLoaderInner {
    pub(super) fn dispatch_io(
        self: &Arc<Self>,
        budget: RenderArtifactManifestIoDispatchBudget,
    ) -> Result<RenderArtifactManifestIoDispatchReport, RenderArtifactManifestIoDispatchError> {
        if budget.max_tasks() == 0 {
            return Err(RenderArtifactManifestIoDispatchError::ZeroTaskLimit);
        }

        self.maintain_deadlines(Instant::now());

        let mut registry = self.lock_registry();
        if !registry.accepting {
            return Err(RenderArtifactManifestIoDispatchError::Closed);
        }
        let mut report = RenderArtifactManifestIoDispatchReport::default();
        let scope = self.scope.census();
        let active_tasks = scope.queued.saturating_add(scope.running);
        let available_task_capacity = scope.task_capacity.saturating_sub(active_tasks);
        let task_limit = budget.max_tasks().min(available_task_capacity);
        while report.submitted_tasks < task_limit {
            let Some((frontier_key, key)) = registry.io_frontier.pop_highest() else {
                break;
            };
            let Some(entry) = registry.entries.get(&key).cloned() else {
                continue;
            };
            let task_id = match take_task_id(&mut registry) {
                Ok(task_id) => task_id,
                Err(error) => {
                    registry.io_frontier.restore(frontier_key, key);
                    return Err(error);
                }
            };
            let entry_for_work = Arc::clone(&entry);
            let store = self.store.clone();
            let store_limits = self.limits.store_limits();
            let metrics = Arc::clone(&self.metrics);
            let task = self.scope.submit(
                TaskDescriptor::new(
                    TaskId::new(task_id),
                    TaskPoolKind::Io,
                    "render-artifact-manifest-read",
                )
                .with_cancellation_policy(TaskCancellationPolicy::CancelOnDrop),
                move |cancellation| {
                    run_manifest_io_task(
                        entry_for_work,
                        store,
                        store_limits,
                        metrics,
                        cancellation,
                    );
                },
            );
            let task = match task {
                Ok(task) => task,
                Err(error) => {
                    registry.io_frontier.restore(frontier_key, key);
                    return Err(error.into());
                }
            };
            entry.install_task(task);
            self.metrics
                .submitted_io_tasks
                .fetch_add(1, Ordering::Relaxed);
            report.submitted_tasks = report.submitted_tasks.saturating_add(1);
        }
        report.remaining_queued_entries = registry.io_frontier.queued_len();
        report.budget_exhausted =
            report.remaining_queued_entries > 0 && report.submitted_tasks == task_limit;
        Ok(report)
    }
}
