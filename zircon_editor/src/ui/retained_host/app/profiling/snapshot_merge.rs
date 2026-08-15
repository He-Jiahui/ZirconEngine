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
    editor_profile.session_id =
        merged_session_id(&editor_profile.session_id, &runtime_profile.session_id);
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

fn merged_session_id(editor_session_id: &str, runtime_session_id: &str) -> String {
    if editor_session_id == runtime_session_id {
        editor_session_id.to_string()
    } else {
        format!("{editor_session_id}+{runtime_session_id}")
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{
        ProfileRecorderRetentionSnapshot, ProfileSampleRetentionSnapshot, ProfileSnapshot,
    };

    use super::merge_profile_snapshot;

    #[test]
    fn merge_preserves_each_recorder_retention_authority() {
        let editor_retention = retention(4, 0);
        let runtime_retention = retention(8, 1);
        let mut editor = ProfileSnapshot {
            active: false,
            recorder_retention: vec![editor_retention.clone()],
            ..ProfileSnapshot::default()
        };
        let runtime = ProfileSnapshot {
            active: true,
            recorder_retention: vec![runtime_retention.clone()],
            ..ProfileSnapshot::default()
        };

        merge_profile_snapshot(&mut editor, runtime);

        assert_eq!(
            editor.recorder_retention,
            vec![editor_retention, runtime_retention]
        );
    }

    fn retention(written: u64, overwritten: u64) -> ProfileRecorderRetentionSnapshot {
        let samples = ProfileSampleRetentionSnapshot {
            capacity: 16,
            written,
            overwritten,
            retained: written.saturating_sub(overwritten),
            oldest_sequence: (written > overwritten).then_some(overwritten),
            newest_sequence: written.checked_sub(1),
        };
        ProfileRecorderRetentionSnapshot {
            frames: samples.clone(),
            spans: samples.clone(),
            counters: samples,
        }
    }
}
