use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use zircon_runtime_interface::{
    ProfileCaptureConfig, ProfileCounterSnapshot, ProfileFrameSnapshot,
    ProfileRecorderRetentionSnapshot, ProfileSampleRetentionSnapshot, ProfileSnapshot,
    ProfileSpanSnapshot,
};

#[derive(Clone, Copy, Debug, Default)]
struct RetentionCounter {
    written: u64,
    overwritten: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileRecorderStatus {
    pub active: bool,
    pub feature_enabled: bool,
    pub message: String,
}

impl ProfileRecorderStatus {
    pub fn disabled() -> Self {
        Self {
            active: false,
            feature_enabled: false,
            message: "profiling feature is disabled".to_string(),
        }
    }
}

/// Single-process ring-buffer timeline recorder used by profiling builds only.
#[derive(Debug)]
pub struct ProfileRecorder {
    config: ProfileCaptureConfig,
    active: bool,
    origin: Instant,
    next_span_id: u64,
    next_frame_index_by_stream: HashMap<String, u64>,
    frames: VecDeque<ProfileFrameSnapshot>,
    spans: VecDeque<ProfileSpanSnapshot>,
    counters: VecDeque<ProfileCounterSnapshot>,
    frame_retention: RetentionCounter,
    span_retention: RetentionCounter,
    counter_retention: RetentionCounter,
}

impl ProfileRecorder {
    pub fn new(config: ProfileCaptureConfig) -> Self {
        Self {
            config: config.normalized(),
            active: false,
            origin: Instant::now(),
            next_span_id: 1,
            next_frame_index_by_stream: HashMap::new(),
            frames: VecDeque::new(),
            spans: VecDeque::new(),
            counters: VecDeque::new(),
            frame_retention: RetentionCounter::default(),
            span_retention: RetentionCounter::default(),
            counter_retention: RetentionCounter::default(),
        }
    }

    pub fn start_capture(&mut self, config: ProfileCaptureConfig) -> ProfileRecorderStatus {
        self.config = config.normalized();
        self.active = true;
        self.origin = Instant::now();
        self.next_span_id = 1;
        self.next_frame_index_by_stream.clear();
        self.frames.clear();
        self.spans.clear();
        self.counters.clear();
        self.reset_retention();
        ProfileRecorderStatus {
            active: true,
            feature_enabled: true,
            message: "profile capture started".to_string(),
        }
    }

    pub fn stop_capture(&mut self) -> ProfileRecorderStatus {
        self.active = false;
        ProfileRecorderStatus {
            active: false,
            feature_enabled: true,
            message: "profile capture stopped".to_string(),
        }
    }

    pub fn reset(&mut self) -> ProfileRecorderStatus {
        self.active = false;
        self.origin = Instant::now();
        self.next_span_id = 1;
        self.next_frame_index_by_stream.clear();
        self.frames.clear();
        self.spans.clear();
        self.counters.clear();
        self.reset_retention();
        ProfileRecorderStatus {
            active: false,
            feature_enabled: true,
            message: "profile capture reset".to_string(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn config(&self) -> &ProfileCaptureConfig {
        &self.config
    }

    pub fn now_us(&self) -> u64 {
        self.origin.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
    }

    pub fn next_span_id(&mut self) -> u64 {
        let id = self.next_span_id;
        self.next_span_id = self.next_span_id.saturating_add(1);
        id
    }

    pub fn next_frame_index(&mut self, stream: &str) -> u64 {
        if let Some(next) = self.next_frame_index_by_stream.get_mut(stream) {
            let frame_index = *next;
            *next = next.saturating_add(1);
            return frame_index;
        }
        self.next_frame_index_by_stream.insert(stream.to_owned(), 1);
        0
    }

    pub fn record_span(&mut self, span: ProfileSpanSnapshot) {
        record_ring_sample(
            &mut self.spans,
            span,
            self.config.max_spans,
            &mut self.span_retention,
        );
    }

    pub fn record_frame(&mut self, frame: ProfileFrameSnapshot) {
        record_ring_sample(
            &mut self.frames,
            frame,
            self.config.max_frames,
            &mut self.frame_retention,
        );
    }

    pub fn record_counter(&mut self, counter: ProfileCounterSnapshot) {
        record_ring_sample(
            &mut self.counters,
            counter,
            self.config.max_counters,
            &mut self.counter_retention,
        );
    }

    pub fn snapshot(&self) -> ProfileSnapshot {
        ProfileSnapshot {
            session_id: self.config.session_id.clone(),
            output_root: self.config.output_root.clone(),
            active: self.active,
            feature_enabled: crate::core::diagnostics::profiling::feature_enabled(),
            frame_budget_ms: self.config.frame_budget_ms,
            frames: self.frames.iter().cloned().collect(),
            spans: self.spans.iter().cloned().collect(),
            counters: self.counters.iter().cloned().collect(),
            recorder_retention: vec![ProfileRecorderRetentionSnapshot {
                frames: retention_snapshot(
                    self.frame_retention,
                    self.frames.len(),
                    self.config.max_frames,
                ),
                spans: retention_snapshot(
                    self.span_retention,
                    self.spans.len(),
                    self.config.max_spans,
                ),
                counters: retention_snapshot(
                    self.counter_retention,
                    self.counters.len(),
                    self.config.max_counters,
                ),
            }],
        }
    }

    fn reset_retention(&mut self) {
        self.frame_retention = RetentionCounter::default();
        self.span_retention = RetentionCounter::default();
        self.counter_retention = RetentionCounter::default();
    }
}

/// Appends to a bounded sample queue without shifting retained samples on eviction.
fn push_ring<T>(items: &mut VecDeque<T>, item: T, max_items: usize) -> bool {
    let max_items = max_items.max(1);
    let overwritten = if items.len() >= max_items {
        items.pop_front();
        true
    } else {
        false
    };
    items.push_back(item);
    overwritten
}

fn record_ring_sample<T>(
    items: &mut VecDeque<T>,
    item: T,
    max_items: usize,
    retention: &mut RetentionCounter,
) {
    retention.written = retention.written.saturating_add(1);
    if push_ring(items, item, max_items) {
        retention.overwritten = retention.overwritten.saturating_add(1);
    }
}

fn retention_snapshot(
    retention: RetentionCounter,
    retained: usize,
    capacity: usize,
) -> ProfileSampleRetentionSnapshot {
    let retained = u64::try_from(retained).unwrap_or(u64::MAX);
    ProfileSampleRetentionSnapshot {
        capacity: u64::try_from(capacity.max(1)).unwrap_or(u64::MAX),
        written: retention.written,
        overwritten: retention.overwritten,
        retained,
        oldest_sequence: (retained > 0).then(|| retention.written.saturating_sub(retained)),
        newest_sequence: (retained > 0).then(|| retention.written.saturating_sub(1)),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use zircon_runtime_interface::{
        ProfileCaptureConfig, ProfileCounterSnapshot, ProfileFrameSnapshot, ProfileSpanSnapshot,
    };

    use super::{ProfileRecorder, push_ring};

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
}
