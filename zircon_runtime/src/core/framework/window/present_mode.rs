use serde::{Deserialize, Serialize};

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
