use serde::{Deserialize, Serialize};

// BUG: [CR-FRAMEWORK-WINDOW-0001] 启动描述符允许选择呈现模式，但当前表面创建固定 Fifo，非 Fifo 请求不会生效；证据见 App 窗口属性转换及 RenderBackend::create_viewport_surface。
/// 启动描述符中的呈现偏好；真正的交换链模式由图形表面协商决定。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowPresentMode {
    AutoVsync,
    AutoNoVsync,
    #[default]
    Fifo,
    FifoRelaxed,
    Immediate,
    Mailbox,
}
