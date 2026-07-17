use serde::{Deserialize, Serialize};

use crate::core::framework::input::InputManager;
use crate::input::{InputEvent, InputEventRecord, InputFrameSnapshot};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InputRecording {
    frames: Vec<InputRecordingFrame>,
}

impl InputRecording {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_frames(frames: Vec<InputRecordingFrame>) -> Self {
        Self { frames }
    }

    pub fn push_frame(&mut self, frame: InputRecordingFrame) {
        self.frames.push(frame);
    }

    pub fn push_captured_frame(&mut self, frame_index: u64, input_manager: &dyn InputManager) {
        self.push_frame(InputRecordingFrame::capture_from_manager(
            frame_index,
            input_manager,
        ));
    }

    pub fn frames(&self) -> &[InputRecordingFrame] {
        &self.frames
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn event_count(&self) -> usize {
        self.frames
            .iter()
            .map(InputRecordingFrame::event_count)
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn discarded_record_count(&self) -> u64 {
        self.frames
            .iter()
            .map(InputRecordingFrame::discarded_record_count)
            .max()
            .unwrap_or(0)
    }

    pub fn is_complete(&self) -> bool {
        self.frames.iter().all(InputRecordingFrame::is_complete)
    }

    pub fn replay_cursor(&self) -> InputReplayCursor<'_> {
        InputReplayCursor::new(self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputRecordingFrame {
    frame_index: u64,
    records: Vec<InputEventRecord>,
    #[serde(default = "recording_enabled_by_default")]
    recording_enabled: bool,
    #[serde(default)]
    discarded_record_count: u64,
}

impl Default for InputRecordingFrame {
    fn default() -> Self {
        Self::new(0, Vec::new())
    }
}

impl InputRecordingFrame {
    pub fn new(frame_index: u64, records: Vec<InputEventRecord>) -> Self {
        Self {
            frame_index,
            records,
            recording_enabled: true,
            discarded_record_count: 0,
        }
    }

    pub fn from_events(frame_index: u64, events: impl IntoIterator<Item = InputEvent>) -> Self {
        let records = events
            .into_iter()
            .enumerate()
            .map(|(index, event)| InputEventRecord {
                sequence: index as u64 + 1,
                timestamp_millis: 0,
                event,
            })
            .collect();
        Self::new(frame_index, records)
    }

    pub fn capture_from_manager(frame_index: u64, input_manager: &dyn InputManager) -> Self {
        let (records, status) = input_manager.drain_event_records_with_status();
        Self {
            frame_index,
            records,
            recording_enabled: status.enabled,
            discarded_record_count: status.discarded_records,
        }
    }

    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub fn records(&self) -> &[InputEventRecord] {
        &self.records
    }

    pub fn event_count(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn discarded_record_count(&self) -> u64 {
        self.discarded_record_count
    }

    pub fn recording_enabled(&self) -> bool {
        self.recording_enabled
    }

    pub fn is_complete(&self) -> bool {
        self.recording_enabled && self.discarded_record_count == 0
    }
}

const fn recording_enabled_by_default() -> bool {
    true
}

#[derive(Debug)]
pub struct InputReplayCursor<'a> {
    recording: &'a InputRecording,
    next_frame: usize,
}

impl<'a> InputReplayCursor<'a> {
    pub fn new(recording: &'a InputRecording) -> Self {
        Self {
            recording,
            next_frame: 0,
        }
    }

    pub fn next_recording_frame_index(&self) -> Option<u64> {
        self.recording
            .frames
            .get(self.next_frame)
            .map(InputRecordingFrame::frame_index)
    }

    pub fn replay_next_frame(
        &mut self,
        input_manager: &dyn InputManager,
    ) -> Option<InputReplayFrameReport> {
        self.replay_next_frame_inner(input_manager, true)
    }

    pub fn submit_next_frame_events(
        &mut self,
        input_manager: &dyn InputManager,
    ) -> Option<InputReplayFrameReport> {
        self.replay_next_frame_inner(input_manager, false)
    }

    pub fn is_finished(&self) -> bool {
        self.next_frame >= self.recording.frames.len()
    }

    fn replay_next_frame_inner(
        &mut self,
        input_manager: &dyn InputManager,
        begin_frame: bool,
    ) -> Option<InputReplayFrameReport> {
        let frame = self.recording.frames.get(self.next_frame)?;
        self.next_frame += 1;
        if begin_frame {
            input_manager.begin_frame();
        }
        for record in frame.records() {
            input_manager.submit_event(record.event.clone());
        }
        Some(InputReplayFrameReport {
            frame_index: frame.frame_index(),
            event_count: frame.event_count(),
            snapshot: input_manager.frame_snapshot(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputReplayFrameReport {
    pub frame_index: u64,
    pub event_count: usize,
    pub snapshot: InputFrameSnapshot,
}
