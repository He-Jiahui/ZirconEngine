//! 滚轮单位与滚量送入 Runtime；有宿主缓存位置时一并编码进 ABI，实际位置消费由会话实现决定。

use winit::event::MouseScrollDelta;
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::{converters::mouse_wheel_delta, RuntimeEntryApp};

impl RuntimeEntryApp {
    // TODO: [CR-APP-ENTRY-0019] 确认滚轮显式位置的消费合同；当前动态会话仅解包滚量，UI 使用会话缓存 cursor，而独立 window adapter 消费 ABI 位置；下一步核验两处缓存不一致或合成事件时的命中语义。
    pub(in crate::entry::runtime_entry_app) fn handle_mouse_wheel(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        delta: MouseScrollDelta,
    ) {
        let (unit, x, y) = mouse_wheel_delta(delta);
        let event = if let Some(position) = self.last_pointer_position {
            ZrRuntimeEventV1::mouse_wheel_delta_at(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                unit,
                position.x as f32,
                position.y as f32,
                x,
                y,
            )
        } else {
            ZrRuntimeEventV1::mouse_wheel_delta(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                unit,
                x,
                y,
            )
        };
        self.dispatch_runtime_event(event_loop, event);
    }
}
