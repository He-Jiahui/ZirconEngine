use super::*;
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

#[cfg(feature = "profiling")]
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ProfileSnapshot, ProfileSpanSnapshot,
};

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
