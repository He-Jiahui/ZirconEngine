use super::super::*;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, write_error, DiagnosticLogLevel,
};

impl RetainedEditorHost {
    pub(super) fn submit_render_frame_if_dirty(&mut self) {
        if !self.render_dirty {
            return;
        }

        let pending_render = self
            .invalidation
            .consume_recompute_reasons(HostInvalidationMask::RENDER);
        let render_reasons = if pending_render.is_empty() {
            HostInvalidationMask::RENDER
        } else {
            pending_render
        };
        let render_rebuild = self.invalidation.record_render_rebuild();
        record_current_ui_perf_counter(UiPerfCounter::RenderPathCount, 1.0);
        self.publish_refresh_invalidation_diagnostics();
        if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
            write_diagnostic_log(
                "editor_host_invalidation",
                format!(
                    "render_path count={} reasons={} {}",
                    render_rebuild,
                    render_reasons.summary(),
                    self.invalidation.stats_summary()
                ),
            );
        }
        let mut keep_render_dirty = false;
        if let Some(submission) = self.runtime.render_frame_submission() {
            zircon_runtime::profile_scope!("editor", "retained_host", "submit_viewport_extract");
            match self.viewport.submit_extract_with_ui(
                submission.extract,
                submission.ui,
                self.viewport_size,
            ) {
                Ok(true) => {
                    let visible_spatial_snapshot = match self.viewport.visible_spatial_snapshot() {
                        Ok(snapshot) => snapshot,
                        Err(error) => {
                            write_diagnostic_log(
                                "editor_viewport_visible_spatial_query",
                                format!("renderer-visible spatial query unavailable: {error}"),
                            );
                            None
                        }
                    };
                    self.runtime
                        .sync_renderer_visible_spatial_snapshot(visible_spatial_snapshot);
                    self.schedule_runtime_diagnostics_refresh();
                }
                Ok(false) => {
                    keep_render_dirty = true;
                }
                Err(error) => {
                    write_error(
                        "editor_viewport_submission",
                        format!("Viewport submit failed: {error}"),
                    );
                    self.set_status_line(format!("Viewport submit failed: {error}"));
                }
            }
        }
        self.render_dirty = keep_render_dirty;
        if keep_render_dirty {
            // Lazy viewport backend startup completes off-thread; queue a
            // non-reentrant frame update so the next redraw can submit the
            // extract once the backend is ready.
            let frame = self.ui.get_host_window_bootstrap().viewport_content_frame;
            self.ui.request_frame_update_region(frame);
        }
    }

    fn schedule_runtime_diagnostics_refresh(&mut self) {
        // Consume the publication-time target so repeated render submissions coalesce behind
        // the pending presentation pass without rescanning the workbench or cloning pane ids.
        match std::mem::take(&mut self.runtime_diagnostics_refresh_target) {
            RuntimeDiagnosticsRefreshTarget::None => {}
            RuntimeDiagnosticsRefreshTarget::Pending => {
                self.runtime_diagnostics_refresh_target = RuntimeDiagnosticsRefreshTarget::Pending;
            }
            RuntimeDiagnosticsRefreshTarget::ShellContent(scope) => {
                self.runtime_diagnostics_refresh_target = RuntimeDiagnosticsRefreshTarget::Pending;
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.runtime_diagnostics.shell_content_refresh_count",
                    1
                );
                self.invalidate_host_for_shell_content(scope, HostInvalidationMask::SHELL_CONTENT);
            }
            RuntimeDiagnosticsRefreshTarget::FullPresentation => {
                self.runtime_diagnostics_refresh_target = RuntimeDiagnosticsRefreshTarget::Pending;
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.runtime_diagnostics.full_presentation_fallback_count",
                    1
                );
                self.mark_presentation_dirty();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn successful_render_submission_refreshes_post_submit_diagnostics_without_requeueing_render() {
        let source = include_str!("render_submission.rs");
        let success_arm = source
            .split_once("Ok(true) => {")
            .and_then(|(_, tail)| tail.split_once("Ok(false) =>"))
            .map(|(arm, _)| arm)
            .expect("render submission success arm should remain explicit");

        assert!(success_arm.contains("self.schedule_runtime_diagnostics_refresh();"));
        assert!(success_arm.contains("visible_spatial_snapshot"));
        assert!(success_arm.contains("sync_renderer_visible_spatial_snapshot"));
        assert!(!success_arm.contains("mark_render_and_presentation_dirty"));
    }

    #[test]
    fn diagnostics_refresh_consumes_a_publication_time_target_without_a_hot_path_scan() {
        let source = include_str!("render_submission.rs");
        let function = source
            .split("fn schedule_runtime_diagnostics_refresh")
            .nth(1)
            .and_then(|tail| tail.split("#[cfg(test)]").next())
            .expect("runtime diagnostics refresh scheduler");

        assert!(function.contains("std::mem::take"));
        assert!(function.contains("RuntimeDiagnosticsRefreshTarget::Pending"));
        assert!(function.contains("RuntimeDiagnosticsRefreshTarget::ShellContent(scope)"));
        assert!(function.contains("HostInvalidationMask::SHELL_CONTENT"));
        assert!(function.contains("RuntimeDiagnosticsRefreshTarget::FullPresentation"));
        assert!(function.contains("self.mark_presentation_dirty();"));
        assert!(
            function
                .find("RuntimeDiagnosticsRefreshTarget::Pending;")
                .unwrap()
                < function
                    .find("self.invalidate_host_for_shell_content")
                    .unwrap()
        );
        assert!(!function.contains("tool_windows.iter"));
        assert!(!function.contains("document_tabs.iter"));
        assert!(!function.contains("floating_windows.iter"));
    }

    #[test]
    fn failed_render_submission_records_the_typed_error_in_process_diagnostics() {
        let source = include_str!("render_submission.rs");
        let error_arm = source
            .split_once("Err(error) => {")
            .and_then(|(_, tail)| tail.split_once("}\n            }"))
            .map(|(arm, _)| arm)
            .expect("render submission error arm should remain explicit");

        assert!(error_arm.contains("write_error("));
        assert!(error_arm.contains("editor_viewport_submission"));
        assert!(error_arm.contains("{error}"));
    }
}
