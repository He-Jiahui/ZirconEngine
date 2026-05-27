use crate::core::ChannelReceiver;

mod button_input_state;
mod file_drag_drop;
mod gamepad;
mod ime;
mod input_button;
mod input_event;
mod input_event_record;
mod input_frame_snapshot;
mod input_snapshot;
mod mouse_wheel;
mod touch;
mod window_status;

pub use button_input_state::ButtonInputState;
pub use file_drag_drop::FileDragDropEvent;
pub use gamepad::{
    GamepadAxis, GamepadAxisSettings, GamepadAxisState, GamepadButton, GamepadButtonAxisSettings,
    GamepadButtonSettings, GamepadButtonValueState, GamepadConnectionInfo, GamepadId,
    GamepadRumbleIntensity, GamepadRumbleRequest, GAMEPAD_AXIS_CHANGE_THRESHOLD,
    GAMEPAD_AXIS_DEADZONE_LOWER, GAMEPAD_AXIS_DEADZONE_UPPER, GAMEPAD_AXIS_LIVEZONE_LOWER,
    GAMEPAD_AXIS_LIVEZONE_UPPER, GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD, GAMEPAD_BUTTON_AXIS_HIGH,
    GAMEPAD_BUTTON_AXIS_LOW, GAMEPAD_BUTTON_PRESS_THRESHOLD, GAMEPAD_BUTTON_RELEASE_THRESHOLD,
};
pub use ime::{
    ImeCursorArea, ImeCursorRange, ImeDeleteSurrounding, ImeEvent, ImeHostRequest, ImePreedit,
    ImeSurroundingText,
};
pub use input_button::InputButton;
pub use input_event::InputEvent;
pub use input_event_record::InputEventRecord;
pub use input_frame_snapshot::InputFrameSnapshot;
pub use input_snapshot::InputSnapshot;
pub use mouse_wheel::{MouseScrollUnit, MouseWheelEvent, LEGACY_PIXEL_SCROLL_SCALE};
pub use touch::{TouchPhase, TouchPoint};
pub use window_status::{WindowStatusEvent, WindowTheme};

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
    fn drain_events(&self) -> Vec<InputEvent>;
    fn drain_event_records(&self) -> Vec<InputEventRecord>;

    fn subscribe_events(&self) -> Option<ChannelReceiver<InputEventRecord>> {
        None
    }
}
