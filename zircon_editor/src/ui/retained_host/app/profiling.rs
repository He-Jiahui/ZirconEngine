use super::*;
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

#[cfg(feature = "profiling")]
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ProfileSnapshot, ProfileSpanSnapshot,
};

pub(super) const PERFORMANCE_TIMELINE_ACTION_CONTROL_ID: &str = "PerformanceTimelineCaptureControl";

#[cfg(feature = "profiling")]
const PERFORMANCE_TIMELINE_START_CAPTURE_ACTION: &str = "PerformanceTimeline.StartCapture";
#[cfg(feature = "profiling")]
const PERFORMANCE_TIMELINE_STOP_CAPTURE_ACTION: &str = "PerformanceTimeline.StopCapture";
#[cfg(feature = "profiling")]
const PERFORMANCE_TIMELINE_EXPORT_REPORT_ACTION: &str = "PerformanceTimeline.ExportReport";
#[cfg(feature = "profiling")]
const PERFORMANCE_TIMELINE_RESET_ACTION: &str = "PerformanceTimeline.Reset";

impl RetainedEditorHost {
    pub(super) fn runtime_diagnostics_with_profile(&self) -> RuntimeDiagnosticsSnapshot {
        #[cfg(feature = "profiling")]
        {
            let mut diagnostics = self.editor_manager.runtime_diagnostics();
            self.merge_dynamic_runtime_profile(&mut diagnostics.profile);
            diagnostics
        }

        #[cfg(not(feature = "profiling"))]
        {
            self.editor_manager.runtime_diagnostics()
        }
    }

    pub(super) fn dispatch_performance_timeline_action(&mut self, action_id: &str) {
        #[cfg(feature = "profiling")]
        {
            self.dispatch_performance_timeline_action_enabled(action_id);
        }

        #[cfg(not(feature = "profiling"))]
        {
            let _ = action_id;
            self.set_status_line("Profiling controls require a profiling build");
        }
    }

    #[cfg(feature = "profiling")]
    fn dispatch_performance_timeline_action_enabled(&mut self, action_id: &str) {
        let Some(command) = profile_command_for_action(action_id) else {
            self.set_status_line(format!("Unknown performance timeline action {action_id}"));
            return;
        };
        let request = ProfileControlRequest {
            command,
            config: None,
        };
        let editor_response =
            zircon_runtime::core::diagnostics::profiling::control(request.clone());
        let runtime_response = self.runtime_client.profile_control(&request);

        self.set_status_line(performance_timeline_action_status(
            &editor_response,
            runtime_response,
        ));
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    #[cfg(feature = "profiling")]
    fn merge_dynamic_runtime_profile(&self, editor_profile: &mut ProfileSnapshot) {
        let request = ProfileControlRequest {
            command: ProfileControlCommand::Snapshot,
            config: None,
        };
        let Ok(Some(response)) = self.runtime_client.profile_control(&request) else {
            return;
        };
        let Some(runtime_profile) = response.snapshot else {
            return;
        };
        merge_profile_snapshot(editor_profile, runtime_profile);
    }
}

#[cfg(feature = "profiling")]
fn profile_command_for_action(action_id: &str) -> Option<ProfileControlCommand> {
    match action_id {
        PERFORMANCE_TIMELINE_START_CAPTURE_ACTION => Some(ProfileControlCommand::StartCapture),
        PERFORMANCE_TIMELINE_STOP_CAPTURE_ACTION => Some(ProfileControlCommand::StopCapture),
        PERFORMANCE_TIMELINE_EXPORT_REPORT_ACTION => Some(ProfileControlCommand::ExportReport),
        PERFORMANCE_TIMELINE_RESET_ACTION => Some(ProfileControlCommand::Reset),
        _ => None,
    }
}

#[cfg(feature = "profiling")]
fn performance_timeline_action_status(
    editor_response: &zircon_runtime_interface::ProfileControlResponse,
    runtime_response: Result<Option<zircon_runtime_interface::ProfileControlResponse>, String>,
) -> String {
    let mut parts = vec![format!("Editor profiling: {}", editor_response.message)];
    match runtime_response {
        Ok(Some(response)) => parts.push(format!("Runtime profiling: {}", response.message)),
        Ok(None) => parts.push("Runtime profiling: unavailable".to_string()),
        Err(error) => parts.push(format!("Runtime profiling: {error}")),
    }
    parts.join("; ")
}

#[cfg(feature = "profiling")]
fn merge_profile_snapshot(
    editor_profile: &mut ProfileSnapshot,
    mut runtime_profile: ProfileSnapshot,
) {
    let editor_has_samples = has_profile_samples(editor_profile);
    if !editor_has_samples && !editor_profile.active {
        *editor_profile = runtime_profile;
        return;
    }

    let span_id_offset = editor_profile
        .spans
        .iter()
        .map(|span| span.id)
        .max()
        .unwrap_or(0);
    if span_id_offset > 0 {
        remap_span_ids(&mut runtime_profile.spans, span_id_offset);
    }

    editor_profile.active |= runtime_profile.active;
    editor_profile.feature_enabled |= runtime_profile.feature_enabled;
    editor_profile.session_id =
        merged_session_id(&editor_profile.session_id, &runtime_profile.session_id);
    editor_profile.frames.extend(runtime_profile.frames);
    editor_profile.spans.extend(runtime_profile.spans);
    editor_profile.counters.extend(runtime_profile.counters);
}

#[cfg(feature = "profiling")]
fn has_profile_samples(profile: &ProfileSnapshot) -> bool {
    !profile.frames.is_empty() || !profile.spans.is_empty() || !profile.counters.is_empty()
}

#[cfg(feature = "profiling")]
fn remap_span_ids(spans: &mut [ProfileSpanSnapshot], offset: u64) {
    for span in spans {
        span.id = span.id.saturating_add(offset);
        span.parent_id = span.parent_id.map(|parent| parent.saturating_add(offset));
    }
}

#[cfg(feature = "profiling")]
fn merged_session_id(editor_session_id: &str, runtime_session_id: &str) -> String {
    if editor_session_id == runtime_session_id {
        editor_session_id.to_string()
    } else {
        format!("{editor_session_id}+{runtime_session_id}")
    }
}

#[cfg(all(test, feature = "profiling"))]
mod tests {
    use super::*;

    #[test]
    fn performance_timeline_actions_map_to_profile_control_commands() {
        assert_eq!(
            profile_command_for_action("PerformanceTimeline.StartCapture"),
            Some(ProfileControlCommand::StartCapture)
        );
        assert_eq!(
            profile_command_for_action("PerformanceTimeline.StopCapture"),
            Some(ProfileControlCommand::StopCapture)
        );
        assert_eq!(
            profile_command_for_action("PerformanceTimeline.ExportReport"),
            Some(ProfileControlCommand::ExportReport)
        );
        assert_eq!(
            profile_command_for_action("PerformanceTimeline.Reset"),
            Some(ProfileControlCommand::Reset)
        );
        assert_eq!(
            profile_command_for_action("PerformanceTimeline.Unknown"),
            None
        );
    }
}
