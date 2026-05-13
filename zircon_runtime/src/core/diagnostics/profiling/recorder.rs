use std::collections::HashMap;
use std::time::Instant;

use zircon_runtime_interface::{
    ProfileCaptureConfig, ProfileCounterSnapshot, ProfileFrameSnapshot, ProfileSnapshot,
    ProfileSpanSnapshot,
};

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
    frames: Vec<ProfileFrameSnapshot>,
    spans: Vec<ProfileSpanSnapshot>,
    counters: Vec<ProfileCounterSnapshot>,
}

impl ProfileRecorder {
    pub fn new(config: ProfileCaptureConfig) -> Self {
        Self {
            config: config.normalized(),
            active: false,
            origin: Instant::now(),
            next_span_id: 1,
            next_frame_index_by_stream: HashMap::new(),
            frames: Vec::new(),
            spans: Vec::new(),
            counters: Vec::new(),
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
        let next = self
            .next_frame_index_by_stream
            .entry(stream.to_string())
            .or_insert(0);
        let frame_index = *next;
        *next = next.saturating_add(1);
        frame_index
    }

    pub fn record_span(&mut self, span: ProfileSpanSnapshot) {
        push_ring(&mut self.spans, span, self.config.max_spans);
    }

    pub fn record_frame(&mut self, frame: ProfileFrameSnapshot) {
        push_ring(&mut self.frames, frame, self.config.max_frames);
    }

    pub fn record_counter(&mut self, counter: ProfileCounterSnapshot) {
        push_ring(&mut self.counters, counter, self.config.max_counters);
    }

    pub fn snapshot(&self) -> ProfileSnapshot {
        ProfileSnapshot {
            session_id: self.config.session_id.clone(),
            output_root: self.config.output_root.clone(),
            active: self.active,
            feature_enabled: crate::core::diagnostics::profiling::feature_enabled(),
            frame_budget_ms: self.config.frame_budget_ms,
            frames: self.frames.clone(),
            spans: self.spans.clone(),
            counters: self.counters.clone(),
        }
    }
}

fn push_ring<T>(items: &mut Vec<T>, item: T, max_items: usize) {
    let max_items = max_items.max(1);
    if items.len() >= max_items {
        items.remove(0);
    }
    items.push(item);
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{
        ProfileCaptureConfig, ProfileCounterSnapshot, ProfileFrameSnapshot, ProfileSpanSnapshot,
    };

    use super::ProfileRecorder;

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
