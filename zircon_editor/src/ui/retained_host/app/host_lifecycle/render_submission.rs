use super::super::*;
use crate::ui::retained_host::ui_perf::{UiPerfCounter, record_current_ui_perf_counter};
use zircon_runtime::diagnostic_log::{
    DiagnosticLogLevel, diagnostic_log_allows, write_diagnostic_log, write_error,
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
                    // RenderStats are updated by submission after pane payloads were collected.
                    // Refresh presentation data once so diagnostics observe the committed frame.
                    self.mark_presentation_dirty();
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

        assert!(success_arm.contains("self.mark_presentation_dirty();"));
        assert!(!success_arm.contains("mark_render_and_presentation_dirty"));
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
