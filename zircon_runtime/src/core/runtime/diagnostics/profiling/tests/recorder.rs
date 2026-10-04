use std::collections::VecDeque;

use zircon_runtime_interface::{
    ProfileCaptureConfig, ProfileCounterSnapshot, ProfileFrameSnapshot, ProfileSpanSnapshot,
    PROFILE_CAPTURE_MAX_COUNTERS, PROFILE_CAPTURE_MAX_FRAMES, PROFILE_CAPTURE_MAX_SPANS,
};

use super::{push_ring, ProfileRecorder};

#[test]
fn ring_push_evicts_oldest_sample_at_capacity() {
    let mut samples = VecDeque::with_capacity(3);
    let mut overwrites = Vec::new();

    for sample in 0..5 {
        overwrites.push(push_ring(&mut samples, sample, 3));
    }

    assert_eq!(overwrites, vec![false, false, false, true, true]);
    assert_eq!(samples.into_iter().collect::<Vec<_>>(), vec![2, 3, 4]);
}

#[test]
fn recorder_retains_latest_items_with_configured_ring_limits() {
    let mut recorder = ProfileRecorder::new(ProfileCaptureConfig {
        max_frames: 1,
        max_spans: 2,
        max_counters: 1,
        ..ProfileCaptureConfig::default()
    });
    recorder.start_capture(ProfileCaptureConfig {
        max_frames: 1,
        max_spans: 2,
        max_counters: 1,
        ..ProfileCaptureConfig::default()
    });

    recorder.record_frame(frame(0));
    recorder.record_frame(frame(1));
    recorder.record_span(span(1, "first"));
    recorder.record_span(span(2, "second"));
    recorder.record_span(span(3, "third"));
    recorder.record_counter(counter(1.0));
    recorder.record_counter(counter(2.0));

    let snapshot = recorder.snapshot();
    assert_eq!(snapshot.frames.len(), 1);
    assert_eq!(snapshot.frames[0].frame_index, 1);
    assert_eq!(snapshot.spans.len(), 2);
    assert_eq!(snapshot.spans[0].name, "second");
    assert_eq!(snapshot.spans[1].name, "third");
    assert_eq!(snapshot.counters.len(), 1);
    assert_eq!(snapshot.counters[0].value, 2.0);
    assert_eq!(snapshot.recorder_retention.len(), 1);
    let retention = &snapshot.recorder_retention[0];
    assert_eq!(retention.frames.capacity, 1);
    assert_eq!(retention.frames.written, 2);
    assert_eq!(retention.frames.overwritten, 1);
    assert_eq!(retention.frames.retained, 1);
    assert_eq!(retention.frames.oldest_sequence, Some(1));
    assert_eq!(retention.frames.newest_sequence, Some(1));
    assert_eq!(retention.spans.capacity, 2);
    assert_eq!(retention.spans.written, 3);
    assert_eq!(retention.spans.overwritten, 1);
    assert_eq!(retention.spans.retained, 2);
    assert_eq!(retention.spans.oldest_sequence, Some(1));
    assert_eq!(retention.spans.newest_sequence, Some(2));
    assert_eq!(retention.counters.capacity, 1);
    assert_eq!(retention.counters.written, 2);
    assert_eq!(retention.counters.overwritten, 1);
    assert_eq!(retention.counters.retained, 1);
    assert_eq!(retention.counters.oldest_sequence, Some(1));
    assert_eq!(retention.counters.newest_sequence, Some(1));
}

#[test]
fn recorder_reset_clears_retention_sequence_authority() {
    let mut recorder = ProfileRecorder::new(ProfileCaptureConfig::default());
    recorder.start_capture(ProfileCaptureConfig::default());
    recorder.record_frame(frame(0));

    recorder.reset();

    let snapshot = recorder.snapshot();
    assert_eq!(snapshot.recorder_retention.len(), 1);
    let retention = &snapshot.recorder_retention[0].frames;
    assert_eq!(retention.written, 0);
    assert_eq!(retention.overwritten, 0);
    assert_eq!(retention.retained, 0);
    assert_eq!(retention.oldest_sequence, None);
    assert_eq!(retention.newest_sequence, None);
}

#[test]
fn recorder_reports_normalized_hard_limits_as_effective_capacity() {
    let recorder = ProfileRecorder::new(ProfileCaptureConfig {
        max_frames: usize::MAX,
        max_spans: usize::MAX,
        max_counters: usize::MAX,
        ..ProfileCaptureConfig::default()
    });

    let snapshot = recorder.snapshot();
    let retention = snapshot
        .recorder_retention
        .first()
        .expect("recorder snapshot must expose effective retention limits");
    assert_eq!(retention.frames.capacity, PROFILE_CAPTURE_MAX_FRAMES as u64);
    assert_eq!(retention.spans.capacity, PROFILE_CAPTURE_MAX_SPANS as u64);
    assert_eq!(
        retention.counters.capacity,
        PROFILE_CAPTURE_MAX_COUNTERS as u64
    );
}

fn frame(frame_index: u64) -> ProfileFrameSnapshot {
    ProfileFrameSnapshot {
        stream: "runtime".to_string(),
        name: "frame".to_string(),
        frame_index,
        start_us: frame_index,
        duration_us: 1,
        budget_ms: 16.67,
        over_budget: false,
    }
}

fn span(id: u64, name: &str) -> ProfileSpanSnapshot {
    ProfileSpanSnapshot {
        id,
        parent_id: None,
        frame_index: Some(0),
        stream: "runtime".to_string(),
        category: "test".to_string(),
        name: name.to_string(),
        path: format!("runtime/test:{name}"),
        start_us: id,
        duration_us: 1,
        depth: 0,
    }
}

fn counter(value: f64) -> ProfileCounterSnapshot {
    ProfileCounterSnapshot {
        stream: "runtime".to_string(),
        name: "counter".to_string(),
        value,
        timestamp_us: value as u64,
        frame_index: Some(0),
    }
}
