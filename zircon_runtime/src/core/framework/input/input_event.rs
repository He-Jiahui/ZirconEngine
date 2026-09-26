use serde::{Deserialize, Serialize};

use super::{
    CursorHostRequest, FileDragDropEvent, GamepadAxis, GamepadButton, GamepadConnectionInfo,
    GamepadId, GamepadRumbleRequest, ImeEvent, ImeHostRequest, InputButton, MouseWheelEvent,
    TouchPhase, WindowStatusEvent,
};

/// 宿主归一化后提交给输入管理器的事件边界；管理器同时更新设备状态、即时帧队列及可选录制。
/// UI 分发可有独立路由，提交这里并不表示 UI 已处理该事件。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    CursorMoved {
        x: f32,
        y: f32,
    },
    CursorEntered,
    CursorLeft,
    MouseMotion {
        delta_x: f32,
        delta_y: f32,
    },
    ButtonPressed(InputButton),
    ButtonReleased(InputButton),
    WheelScrolled {
        delta: f32,
    },
    MouseWheel(MouseWheelEvent),
    KeyboardInput {
        key_code: u32,
        logical_key: Option<String>,
        text: Option<String>,
        pressed: bool,
        repeat: bool,
    },
    WindowStatus(WindowStatusEvent),
    FileDragDrop(FileDragDropEvent),
    Ime(ImeEvent),
    ImeHostRequest(ImeHostRequest),
    CursorHostRequest(CursorHostRequest),
    FocusLost,
    Touch {
        id: u64,
        phase: TouchPhase,
        x: f32,
        y: f32,
    },
    GamepadConnection(GamepadConnectionInfo),
    GamepadButton {
        gamepad: GamepadId,
        button: GamepadButton,
        value: f32,
        pressed: bool,
    },
    GamepadAxis {
        gamepad: GamepadId,
        axis: GamepadAxis,
        value: f32,
    },
    GamepadRumbleRequest(GamepadRumbleRequest),
}
