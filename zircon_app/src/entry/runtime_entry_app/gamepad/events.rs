//! Gilrs 事件到动态 Runtime 的同步提交。
//! 名称字节借用仅覆盖一次调用；设备编号属于当前 Gilrs 会话，不能作为持久硬件身份。

use gilrs::{Axis, Button};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeGamepadGenerationV2, ZrRuntimeViewportHandle,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1,
    ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1,
};

use super::codes::{axis_code, button_code};
use super::producer::CapturedGamepadStamp;
use crate::entry::runtime_library::{RuntimeLibraryError, RuntimeSession};

/// 同步发送连接清单；名称只在调用期间有效，接收端必须在返回前读取或复制。
pub(super) fn send_connection(
    session: &RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    stamp: CapturedGamepadStamp,
    connected: bool,
    name: &str,
    vendor_id: Option<u16>,
    product_id: Option<u16>,
) -> Result<(), RuntimeLibraryError> {
    let state = if connected {
        ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1
    } else {
        ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1
    };
    let payload = ZrRuntimeGamepadGenerationV2::encode_connection(stamp.generation, Some(name))
        .map_err(|error| {
            RuntimeLibraryError::new(format!("encode gamepad V2 connection: {error:?}"))
        })?;
    let event = ZrRuntimeEventV1::gamepad_connection_v2(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        stamp.gamepad,
        state,
        vendor_id.map(u32::from).unwrap_or_default(),
        product_id.map(u32::from).unwrap_or_default(),
        byte_slice(&payload),
    );
    session.handle_event(event)
}

pub(super) fn send_button(
    session: &RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    stamp: CapturedGamepadStamp,
    button: Button,
    value: f32,
    pressed: bool,
) -> Result<(), RuntimeLibraryError> {
    let state = if pressed {
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1
    } else {
        ZR_RUNTIME_BUTTON_STATE_RELEASED_V1
    };
    let payload = ZrRuntimeGamepadGenerationV2::encode_sample(stamp.generation);
    let event = ZrRuntimeEventV1::gamepad_button_v2(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        stamp.gamepad,
        button_code(button),
        state,
        value,
        byte_slice(&payload),
    );
    session.handle_event(event)
}

// TODO: [CR-APP-ENTRY-0014] 确认原始按钮固定 pressed 码与 UI 导航的消费约定；当前接收端仅按 state 决定导航，需在其外来改动稳定后验证零值与释放不会重复激活。
/// 保留模拟按钮原始值；App 不自行计算阈值，接收端按协议解释。
pub(super) fn send_raw_button(
    session: &RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    stamp: CapturedGamepadStamp,
    button: Button,
    value: f32,
) -> Result<(), RuntimeLibraryError> {
    let payload = ZrRuntimeGamepadGenerationV2::encode_sample(stamp.generation);
    let event = ZrRuntimeEventV1::gamepad_button_v2(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        stamp.gamepad,
        button_code(button),
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
        value,
        byte_slice(&payload),
    );
    session.handle_event(event)
}

pub(super) fn send_axis(
    session: &RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    stamp: CapturedGamepadStamp,
    axis: Axis,
    value: f32,
) -> Result<(), RuntimeLibraryError> {
    let payload = ZrRuntimeGamepadGenerationV2::encode_sample(stamp.generation);
    let event = ZrRuntimeEventV1::gamepad_axis_v2(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        stamp.gamepad,
        axis_code(axis),
        value,
        byte_slice(&payload),
    );
    session.handle_event(event)
}

pub(super) fn gamepad_id(id: gilrs::GamepadId) -> u64 {
    let id: usize = id.into();
    id as u64
}

fn byte_slice(value: &[u8]) -> ZrByteSlice {
    ZrByteSlice {
        data: value.as_ptr(),
        len: value.len(),
    }
}
