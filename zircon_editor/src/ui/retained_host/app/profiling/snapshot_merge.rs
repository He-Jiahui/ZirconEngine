use zircon_runtime_interface::{ProfileSnapshot, ProfileSpanSnapshot};

pub(super) fn merge_profile_snapshot(
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
    merge_session_id_in_place(&mut editor_profile.session_id, &runtime_profile.session_id);
    editor_profile.frames.extend(runtime_profile.frames);
    editor_profile.spans.extend(runtime_profile.spans);
    editor_profile.counters.extend(runtime_profile.counters);
    editor_profile
        .recorder_retention
        .extend(runtime_profile.recorder_retention);
}

fn has_profile_samples(profile: &ProfileSnapshot) -> bool {
    !profile.frames.is_empty()
        || !profile.spans.is_empty()
        || !profile.counters.is_empty()
        || !profile.recorder_retention.is_empty()
}

fn remap_span_ids(spans: &mut [ProfileSpanSnapshot], offset: u64) {
    for span in spans {
        span.id = span.id.saturating_add(offset);
        span.parent_id = span.parent_id.map(|parent| parent.saturating_add(offset));
    }
}

fn merge_session_id_in_place(editor_session_id: &mut String, runtime_session_id: &str) {
    if editor_session_id != runtime_session_id {
        editor_session_id.reserve(1 + runtime_session_id.len());
        editor_session_id.push('+');
        editor_session_id.push_str(runtime_session_id);
    }
}

#[cfg(test)]
#[path = "tests/snapshot_merge.rs"]
mod tests;

#[cfg(test)]
#[path = "snapshot_merge/tests/owned_session_id_tests.rs"]
mod owned_session_id_tests;
