//! 原始设备事件到 App 指针输入的准入层。
//! 只为被 Runtime 消费的设备事件安排响应式帧。

use winit::event::DeviceEvent;
use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: DeviceEvent,
    ) {
        if !device_event_requests_runtime_frame(&event) {
            return;
        }
        self.request_runtime_frame();
        self.handle_pointer_device_event(event_loop, event);
    }
}

fn device_event_requests_runtime_frame(event: &DeviceEvent) -> bool {
    matches!(event, DeviceEvent::PointerMotion { .. })
}

#[cfg(test)]
#[path = "tests/dispatch.rs"]
mod tests;
