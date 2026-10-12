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
