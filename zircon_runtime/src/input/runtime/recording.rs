use serde::{Deserialize, Serialize};

use crate::core::framework::input::InputManager;
use crate::input::{InputEvent, InputEventRecord, InputFrameSnapshot};

/// 按帧持有原始输入记录；采集者可先检查完整性，再由游标把事件重放进任意 InputManager。
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

    /// 返回管理器累计丢弃量的最大观测值；各帧保存累计状态，不能相加作为记录总丢弃数。
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

/// 一帧事件批次及其完整性状态；采集模式由调用者指定帧编号，记录队列在每次采集后排空。
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

    /// 用现有事件合成一帧；序号从一开始、时间戳为零，原始时序须通过带元数据的记录传入。
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

    /// 排空管理器的记录队列并保存当时的丢弃状态；完整回放要求事先启用记录并在帧边界采集。
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

/// 借用不可变记录并按存储顺序推进；帧编号仅作标识，不等待时间戳或补齐缺失的帧编号。
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

    /// 自行开启下一帧后投递记录；宿主已经开启帧时使用 submit_next_frame_events。
    pub fn replay_next_frame(
        &mut self,
        input_manager: &dyn InputManager,
    ) -> Option<InputReplayFrameReport> {
        self.replay_next_frame_inner(input_manager, true)
    }

    /// 向宿主已开启的帧追加下一批记录；调用方须自行清理上一帧边沿和推进其它帧服务。
    pub fn submit_next_frame_events(
        &mut self,
        input_manager: &dyn InputManager,
    ) -> Option<InputReplayFrameReport> {
        self.replay_next_frame_inner(input_manager, false)
    }

    pub fn is_finished(&self) -> bool {
        self.next_frame >= self.recording.frames.len()
    }

    // TODO: [CR-INPUT-0002] 确认不完整帧是否允许默认重放；当前游标不检查 recording_enabled 与丢弃数，缺少缺失事件的回放契约测试；下一步核对宿主调用。
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

/// 一批事件投递后的即时输入快照；与源帧编号关联，完整性仍须在原记录上查询。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputReplayFrameReport {
    pub frame_index: u64,
    pub event_count: usize,
    pub snapshot: InputFrameSnapshot,
}
