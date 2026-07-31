use super::diagnostics::UiAssetWorkspaceWatchPollReport;
use super::service::{UiAssetWatchPollStart, UiAssetWorkspaceWatcher};
use crate::ui::host::editor_error::EditorError;
use crate::ui::host::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub(in crate::ui::host) fn restart_ui_asset_workspace_watcher(
        &self,
    ) -> Result<(), EditorError> {
        let project = self.current_project_snapshot()?;
        let project_root = project.as_ref().map(|project| project.paths().root());
        let transitioned = self
            .lock_ui_asset_refresh_pipeline()
            .transition_project(project_root);
        let Some(project) = project else {
            *self.lock_ui_asset_workspace_watcher() = None;
            return Ok(());
        };
        if !transitioned && self.lock_ui_asset_workspace_watcher().is_some() {
            return Ok(());
        }
        let watcher = UiAssetWorkspaceWatcher::start(&project)?;
        *self.lock_ui_asset_workspace_watcher() = Some(watcher);
        Ok(())
    }

    pub fn poll_ui_asset_workspace_watcher(
        &self,
    ) -> Result<UiAssetWorkspaceWatchPollReport, EditorError> {
        let poll_start = self
            .lock_ui_asset_workspace_watcher()
            .as_mut()
            .map(UiAssetWorkspaceWatcher::begin_poll);
        let mut report = match poll_start {
            None => UiAssetWorkspaceWatchPollReport::default(),
            Some(UiAssetWatchPollStart::Ready(report)) => report,
            Some(UiAssetWatchPollStart::Reconcile {
                mut cursor,
                mut allowance,
            }) => {
                let (changed_asset_ids, completed) =
                    self.collect_ui_asset_reconcile_batch(&mut cursor, &mut allowance);
                let mut watcher = self.lock_ui_asset_workspace_watcher();
                match watcher.as_mut() {
                    Some(watcher) => watcher.finish_reconcile(
                        (!completed).then_some(cursor),
                        allowance,
                        changed_asset_ids,
                    ),
                    None => UiAssetWorkspaceWatchPollReport {
                        changed_asset_ids: changed_asset_ids.into_iter().collect(),
                        diagnostics: Default::default(),
                    },
                }
            }
        };
        self.lock_ui_asset_refresh_pipeline()
            .enqueue(report.changed_asset_ids.iter().cloned());
        // Observe ingress before accepting a worker result so a same-tick
        // filesystem generation always supersedes the older parse.
        self.commit_completed_ui_asset_refresh()?;
        self.start_next_ui_asset_refresh()?;
        let refresh = self.lock_ui_asset_refresh_pipeline().snapshot();
        report.diagnostics.refresh_pending_asset_count = refresh.pending_asset_count;
        report.diagnostics.refresh_active = refresh.active;
        report.diagnostics.refresh_deferred_retry_count = refresh.deferred_retry_count;
        report.diagnostics.refresh_exhausted_retry_count = refresh.exhausted_retry_count;
        report.diagnostics.refresh_superseded_count = refresh.superseded_count;
        Ok(report)
    }

    fn commit_completed_ui_asset_refresh(&self) -> Result<(), EditorError> {
        let completion = self.lock_ui_asset_refresh_pipeline().take_completed();
        let Some(completion) = completion else {
            return Ok(());
        };
        if completion.superseded {
            return Ok(());
        }
        let changed_asset_ids = completion.changed_asset_ids;
        let generation = completion.generation;
        let retry_attempt = completion.retry_attempt;
        let batch = match completion.result {
            Ok(batch) => batch,
            Err(error) => {
                self.lock_ui_asset_refresh_pipeline().defer_retry(
                    changed_asset_ids,
                    retry_attempt,
                    generation,
                );
                return Err(EditorError::UiAsset(error.to_string()));
            }
        };
        let commit = match self.commit_ui_asset_refresh_batch(batch) {
            Ok(commit) => commit,
            Err(error) => {
                self.lock_ui_asset_refresh_pipeline().defer_retry(
                    changed_asset_ids,
                    retry_attempt,
                    generation,
                );
                return Err(error);
            }
        };
        let mut requeue_asset_ids = commit.requeue_asset_ids;
        for asset_id in &commit.retry_asset_ids {
            requeue_asset_ids.remove(asset_id);
        }
        self.lock_ui_asset_refresh_pipeline()
            .enqueue(requeue_asset_ids);
        self.lock_ui_asset_refresh_pipeline().defer_retry(
            commit.retry_asset_ids,
            retry_attempt,
            generation,
        );
        if let Err(error) = self.sync_ui_asset_refresh_instances(commit.sync_instances) {
            self.lock_ui_asset_refresh_pipeline().defer_retry(
                changed_asset_ids,
                retry_attempt,
                generation,
            );
            return Err(error);
        }
        Ok(())
    }

    fn start_next_ui_asset_refresh(&self) -> Result<(), EditorError> {
        let request = self.lock_ui_asset_refresh_pipeline().begin_request();
        let Some(request) = request else {
            return Ok(());
        };
        let retry = request.clone();
        let plan = match self.build_ui_asset_refresh_plan(request) {
            Ok(plan) => plan,
            Err(error) => {
                self.lock_ui_asset_refresh_pipeline().defer_request(retry);
                return Err(error);
            }
        };
        self.lock_ui_asset_refresh_pipeline()
            .submit(plan)
            .map_err(EditorError::UiAsset)
    }
}
