use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::input::{
    InputEvent, InputEventRecord, InputEventRecordingConfig, InputEventRecordingStatus,
};

#[derive(Debug, Default)]
pub(in crate::input::runtime) struct InputEventRecorder {
    config: InputEventRecordingConfig,
    records: VecDeque<InputEventRecord>,
    discarded_records: u64,
    next_sequence: u64,
}

impl InputEventRecorder {
    pub(in crate::input::runtime) fn configure(&mut self, config: InputEventRecordingConfig) {
        if !config.enabled {
            self.config = config;
            self.records.clear();
            self.discarded_records = 0;
            self.next_sequence = 0;
            return;
        }

        if !self.config.enabled {
            self.records.clear();
            self.discarded_records = 0;
            self.next_sequence = 0;
        }
        self.config = config;
        while self.records.len() > config.capacity as usize {
            self.records.pop_front();
            self.discarded_records = self.discarded_records.saturating_add(1);
        }
    }

    pub(in crate::input::runtime) fn record(&mut self, event: &InputEvent) {
        if !self.config.enabled {
            return;
        }

        self.next_sequence = self.next_sequence.saturating_add(1);
        if self.config.capacity == 0 {
            self.discarded_records = self.discarded_records.saturating_add(1);
            return;
        }
        if self.records.len() >= self.config.capacity as usize {
            self.records.pop_front();
            self.discarded_records = self.discarded_records.saturating_add(1);
        }
        self.records.push_back(InputEventRecord {
            sequence: self.next_sequence,
            timestamp_millis: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            event: event.clone(),
        });
    }

    pub(in crate::input::runtime) fn status(&self) -> InputEventRecordingStatus {
        InputEventRecordingStatus {
            enabled: self.config.enabled,
            capacity: self.config.capacity,
            retained_records: self.records.len() as u32,
            discarded_records: self.discarded_records,
        }
    }

    pub(in crate::input::runtime) fn drain(&mut self) -> Vec<InputEventRecord> {
        self.records.drain(..).collect()
    }
}
