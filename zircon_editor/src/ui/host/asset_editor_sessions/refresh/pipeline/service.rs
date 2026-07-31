use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::core::jobs::{
    EditorJobSpec, EditorJobSystem, JobCategory, JobError, JobPriority, JobTicket,
};

use super::job::UiAssetRefreshJob;
use super::plan::UiAssetRefreshPlan;
use super::queue::{UiAssetRefreshQueue, UiAssetRefreshQueueSnapshot, UiAssetRefreshRequest};
use super::result::UiAssetRefreshBatch;

pub(in crate::ui::host::asset_editor_sessions) struct UiAssetRefreshCompletion {
    pub(in crate::ui::host::asset_editor_sessions) result: Result<UiAssetRefreshBatch, JobError>,
    pub(in crate::ui::host::asset_editor_sessions) superseded: bool,
    pub(in crate::ui::host::asset_editor_sessions) changed_asset_ids: BTreeSet<String>,
    pub(in crate::ui::host::asset_editor_sessions) generation: u64,
    pub(in crate::ui::host::asset_editor_sessions) retry_attempt: u8,
}

struct ActiveUiAssetRefresh {
    request: UiAssetRefreshRequest,
    ticket: JobTicket<UiAssetRefreshBatch>,
}

pub(crate) struct UiAssetWorkspaceRefreshPipeline {
    jobs: EditorJobSystem,
    queue: UiAssetRefreshQueue,
    active: Option<ActiveUiAssetRefresh>,
    project_root: Option<PathBuf>,
}

impl UiAssetWorkspaceRefreshPipeline {
    pub(crate) fn new(jobs: EditorJobSystem) -> Self {
        Self {
            jobs,
            queue: UiAssetRefreshQueue::default(),
            active: None,
            project_root: None,
        }
    }

    pub(in crate::ui::host::asset_editor_sessions) fn enqueue(
        &mut self,
        changed_asset_ids: impl IntoIterator<Item = String>,
    ) -> bool {
        self.queue.enqueue(changed_asset_ids)
    }

    pub(in crate::ui::host::asset_editor_sessions) fn begin_request(
        &mut self,
    ) -> Option<UiAssetRefreshRequest> {
        self.queue.start_next()
    }

    pub(in crate::ui::host::asset_editor_sessions) fn submit(
        &mut self,
        plan: UiAssetRefreshPlan,
    ) -> Result<(), String> {
        if plan.is_empty() {
            self.queue.complete_without_work(plan.generation);
            return Ok(());
        }
        let request = plan.request();
        let generation = plan.generation;
        let spec = EditorJobSpec::new(
            format!("Refresh UI asset generation {generation}"),
            JobCategory::Index,
        )
        .with_priority(JobPriority::Background);
        match self.jobs.submit(spec, UiAssetRefreshJob { plan }) {
            Ok(ticket) => {
                self.active = Some(ActiveUiAssetRefresh { request, ticket });
                Ok(())
            }
            Err(error) => {
                self.defer_request_at(request, Instant::now());
                Err(error.to_string())
            }
        }
    }

    pub(in crate::ui::host::asset_editor_sessions) fn defer_request(
        &mut self,
        request: UiAssetRefreshRequest,
    ) {
        self.defer_request_at(request, Instant::now());
    }

    pub(in crate::ui::host::asset_editor_sessions) fn defer_retry(
        &mut self,
        changed_asset_ids: BTreeSet<String>,
        retry_attempt: u8,
        generation: u64,
    ) -> bool {
        self.queue
            .defer_retry_at(changed_asset_ids, retry_attempt, generation, Instant::now())
    }

    pub(in crate::ui::host::asset_editor_sessions) fn transition_project(
        &mut self,
        project_root: Option<&Path>,
    ) -> bool {
        let project_root = project_root.map(Path::to_path_buf);
        if self.project_root == project_root {
            return false;
        }
        self.reset_project_epoch();
        self.project_root = project_root;
        true
    }

    pub(in crate::ui::host::asset_editor_sessions) fn snapshot(
        &self,
    ) -> UiAssetRefreshQueueSnapshot {
        self.queue.snapshot()
    }

    fn reset_project_epoch(&mut self) {
        if let Some(active) = self.active.take() {
            self.jobs.cancel(active.ticket.id());
        }
        self.queue.reset_project_epoch();
    }

    pub(in crate::ui::host::asset_editor_sessions) fn take_completed(
        &mut self,
    ) -> Option<UiAssetRefreshCompletion> {
        let active = self.active.as_ref()?;
        let result = active.ticket.try_take()?;
        let active = self.active.take().expect("active refresh checked above");
        let superseded = self.queue.finish(&active.request);
        Some(UiAssetRefreshCompletion {
            result,
            superseded,
            generation: active.request.generation,
            retry_attempt: active.request.retry_attempt,
            changed_asset_ids: active.request.changed_asset_ids,
        })
    }

    fn defer_request_at(&mut self, request: UiAssetRefreshRequest, now: Instant) {
        if !self.queue.finish(&request) {
            self.queue.defer_retry_at(
                request.changed_asset_ids,
                request.retry_attempt,
                request.generation,
                now,
            );
        }
    }
}
