use crate::core::framework::channel::ChannelReceiver;

use super::{
    ButtonInputState, CursorHostRequest, GamepadRumbleRequest, ImeHostRequest, InputButton,
    InputEvent, InputEventQueueStatus, InputEventRecord, InputEventRecordingConfig,
    InputEventRecordingStatus, InputFrameSnapshot, InputSnapshot, MouseWheelEvent,
};

/// 宿主事件进入运行时输入栈的服务边界；一帧先 `begin_frame`、再提交事件、再读取快照或排空队列。
/// 完整实现同时维护设备持久状态、帧内通知和可选原始记录，调用方须区分这些生命周期。
pub trait InputManager: Send + Sync {
    /// 支持帧状态的实现应清除瞬时边沿与增量，同时保留按住和连接状态。
    fn begin_frame(&self) {}
    fn submit_event(&self, event: InputEvent);
    fn snapshot(&self) -> InputSnapshot;

    fn button_pressed(&self, button: &InputButton) -> bool {
        self.snapshot().pressed_buttons.contains(button)
    }

    /// 返回完整帧视图；仅实现 `snapshot` 的简化管理器会缺少边沿和设备扩展状态。
    fn frame_snapshot(&self) -> InputFrameSnapshot {
        let snapshot = self.snapshot();
        let buttons = ButtonInputState::from_pressed(snapshot.pressed_buttons);
        InputFrameSnapshot {
            cursor_position: snapshot.cursor_position,
            buttons,
            wheel_accumulator: snapshot.wheel_accumulator,
            mouse_wheel_accumulator: [0.0, snapshot.wheel_accumulator],
            mouse_wheel_events: if snapshot.wheel_accumulator == 0.0 {
                Vec::new()
            } else {
                vec![MouseWheelEvent::lines(0.0, snapshot.wheel_accumulator)]
            },
            ..InputFrameSnapshot::default()
        }
    }

    /// 将运行时请求交给宿主执行；排空后本次请求不应再被其他宿主批次发送。
    fn drain_ime_host_requests(&self) -> Vec<ImeHostRequest> {
        Vec::new()
    }

    fn drain_gamepad_rumble_requests(&self) -> Vec<GamepadRumbleRequest> {
        Vec::new()
    }

    fn drain_cursor_host_requests(&self) -> Vec<CursorHostRequest> {
        Vec::new()
    }

    /// 排空用于即时消费的帧事件；它可能合并相邻移动事件，不适合作无损回放源。
    fn drain_events(&self) -> Vec<InputEvent>;
    fn drain_event_records(&self) -> Vec<InputEventRecord>;

    /// Drains records and observes their retention status under one manager transaction.
    /// 在同一管理器事务中取得原始记录及丢弃状态，避免将不完整录制当作无损回放。
    fn drain_event_records_with_status(&self)
        -> (Vec<InputEventRecord>, InputEventRecordingStatus);

    fn set_event_recording_config(&self, _config: InputEventRecordingConfig) {}

    fn event_recording_status(&self) -> InputEventRecordingStatus {
        InputEventRecordingStatus::default()
    }

    fn event_queue_status(&self) -> InputEventQueueStatus {
        InputEventQueueStatus::default()
    }

    fn subscribe_events(&self) -> Option<ChannelReceiver<InputEventRecord>> {
        None
    }
}
