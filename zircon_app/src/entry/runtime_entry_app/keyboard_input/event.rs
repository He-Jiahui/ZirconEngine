//! 目标窗口键盘事件进入动态 Runtime 的同步派发边界。
//! 文本只借用到会话事件调用返回，不能把 ABI 字节视图留在宿主队列中。

use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::{
    converters::{key_action, physical_key_code},
    RuntimeEntryApp,
};
use super::payload::keyboard_text_payload;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_keyboard_input(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: KeyEvent,
    ) {
        if let Some(action) = key_action(event.state) {
            let payload = keyboard_text_payload(&event);
            let runtime_event = ZrRuntimeEventV1::keyboard(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                action,
                physical_key_code(&event.physical_key),
                0,
                payload,
            );
            self.dispatch_runtime_event(event_loop, runtime_event);
        }
    }
}
