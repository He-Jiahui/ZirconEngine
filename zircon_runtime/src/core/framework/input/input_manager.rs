use crate::core::framework::channel::ChannelReceiver;

use super::{
    ButtonInputState, CursorHostRequest, GamepadRumbleRequest, ImeHostRequest, InputEvent,
    InputEventQueueStatus, InputEventRecord, InputEventRecordingConfig, InputEventRecordingStatus,
    InputFrameSnapshot, InputSnapshot, MouseWheelEvent,
};

pub trait InputManager: Send + Sync {
    fn begin_frame(&self) {}
    fn submit_event(&self, event: InputEvent);
    fn snapshot(&self) -> InputSnapshot;

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

    fn drain_ime_host_requests(&self) -> Vec<ImeHostRequest> {
        Vec::new()
    }

    fn drain_gamepad_rumble_requests(&self) -> Vec<GamepadRumbleRequest> {
        Vec::new()
    }

    fn drain_cursor_host_requests(&self) -> Vec<CursorHostRequest> {
        Vec::new()
    }

    fn drain_events(&self) -> Vec<InputEvent>;
    fn drain_event_records(&self) -> Vec<InputEventRecord>;

    /// Drains records and observes their retention status under one manager transaction.
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
